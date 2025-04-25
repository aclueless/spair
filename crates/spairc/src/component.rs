use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    rc::{Rc, Weak},
};

use wasm_bindgen::UnwrapThrowExt;

use crate::routing::{Route, get_current_location, setup_routing};

use crate::element::WsElement;

pub fn start_app<C: Component + 'static>(new_state: impl FnOnce(Comp<C>) -> C) {
    let rc_comp = create_component(new_state, |_, _: ()| {}, |_, _| {});
    std::mem::forget(rc_comp);
}

pub fn start_app_with_routing<C: Component + 'static, R: Route + 'static>(
    new_state: impl FnOnce(Comp<C>) -> C,
    set_route: fn(&mut C, R),
) {
    let rc_comp = create_component(new_state, set_route, setup_routing);
    std::mem::forget(rc_comp);
}

pub(crate) fn create_component<C: Component + 'static, R: Route>(
    new_state: impl FnOnce(Comp<C>) -> C,
    set_route: fn(&mut C, R),
    setup_routing: impl FnOnce(fn(&mut C, R), Comp<C>),
) -> RcComp<C> {
    let comp_data = CompData(None);
    let rc_comp = RcComp(Rc::new(RefCell::new(comp_data)));
    let mut state = new_state(rc_comp.comp());

    let route = R::from_location(&get_current_location());
    set_route(&mut state, route);
    setup_routing(set_route, rc_comp.comp());

    finalize_rc_comp(rc_comp, state)
}

fn finalize_rc_comp<C: Component + 'static>(rc_comp: RcComp<C>, state: C) -> RcComp<C> {
    let comp = rc_comp.comp();
    let context = comp.context(&state);
    let (root, mut view_state) = C::create_view(&context);
    C::update_view(&mut view_state, &context);

    match rc_comp.0.try_borrow_mut() {
        Ok(mut rc_comp) => {
            rc_comp.0 = Some(CompDataInner {
                root,
                state,
                view_state,
            });
        }
        _ => log::error!("Internal error: unable to mutable borrow rc_comp to set store its data"),
    }
    rc_comp
}

thread_local! {
    static UPDATE_QUEUE_IS_IN_EXECUTING: Cell<bool> = const { Cell::new(false) };
    static UPDATE_QUEUE: RefCell<VecDeque<Box<dyn FnOnce()>>> = RefCell::new(VecDeque::new());
}

fn is_update_queue_executing() -> bool {
    UPDATE_QUEUE_IS_IN_EXECUTING.with(|executing| executing.get())
}

fn update_queue_will_be_executing() -> bool {
    UPDATE_QUEUE_IS_IN_EXECUTING.with(|executing| !executing.replace(true))
}

fn put_callback_on_update_queue(callback: impl FnOnce() + 'static) {
    UPDATE_QUEUE.with(|queue| match queue.try_borrow_mut() {
        Ok(mut queue) => queue.push_back(Box::new(callback)),
        Err(e) => {
            log::error!("Error on queuing an update callback {e}");
        }
    });
}

fn execute_update_queue() {
    UPDATE_QUEUE.with(|queue| {
        while let Some(callback) = queue.try_borrow_mut().ok().and_then(|mut v| v.pop_front()) {
            callback();
        }
    });
    UPDATE_QUEUE_IS_IN_EXECUTING.with(|executing| executing.set(false));
}

struct CompData<C: Component>(Option<CompDataInner<C>>);

struct CompDataInner<C: Component> {
    root: WsElement,
    state: C,
    view_state: C::ViewState,
}

pub struct RcComp<C: Component>(Rc<RefCell<CompData<C>>>);
pub struct Comp<C: Component>(Weak<RefCell<CompData<C>>>);

pub struct Context<'a, C: Component> {
    pub comp: &'a Comp<C>,
    pub state: &'a C,
}

impl<C: Component> Clone for Comp<C> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<C: Component> Clone for Context<'_, C> {
    fn clone(&self) -> Self {
        Self {
            comp: self.comp,
            state: self.state,
        }
    }
}

impl<C: Component + 'static> RcComp<C> {
    pub fn comp(&self) -> Comp<C> {
        Comp(Rc::downgrade(&self.0))
    }

    pub fn new(new_state: impl FnOnce(&Comp<C>) -> C) -> Self {
        let comp_data = CompData(None);
        let rc_comp = RcComp(Rc::new(RefCell::new(comp_data)));
        let comp = rc_comp.comp();
        finalize_rc_comp(rc_comp, new_state(&comp))
    }

    pub fn root_element(&self) -> WsElement {
        self.0
            .try_borrow()
            .expect_throw("Error on borrowing RcComp content to get the component's root element")
            .0
            .as_ref()
            .expect_throw("RcComp CompData is empty")
            .root
            .clone()
    }
}

pub enum ShouldRender {
    No,
    Yes,
}

impl From<()> for ShouldRender {
    fn from(_: ()) -> Self {
        ShouldRender::Yes
    }
}

type MutFnArg<C, A> = dyn Fn(&mut C, A) -> ShouldRender;

pub struct CallbackFnArg<C: Component, A> {
    comp: Comp<C>,
    callback: Rc<MutFnArg<C, A>>,
}

impl<C: Component, A> CallbackFnArg<C, A>
where
    C: 'static + Component,
    A: 'static,
{
    fn clone(&self) -> Self {
        Self {
            comp: self.comp.clone(),
            callback: self.callback.clone(),
        }
    }

    pub fn execute_or_queue(&self, arg: A) {
        if is_update_queue_executing() {
            self.queue(arg);
            return;
        }
        self.execute(arg);
    }

    fn execute(&self, arg: A) {
        self.comp
            .execute_given_callback_then_the_update_queue(arg, self);
    }

    fn queue(&self, arg: A) {
        let clone = self.clone();
        put_callback_on_update_queue(move || clone.execute(arg));
    }
}

pub trait CallbackArgTrait<A> {
    fn call(&self, arg: A);
}

pub type CallbackArg<A> = Box<dyn CallbackArgTrait<A>>;

impl<C: Component, A> CallbackArgTrait<A> for CallbackFnArg<C, A>
where
    C: 'static + Component,
    A: 'static,
{
    fn call(&self, arg: A) {
        self.execute_or_queue(arg)
    }
}

pub trait Component: Sized {
    type ViewState;
    fn create_view(ccontext: &Context<Self>) -> (WsElement, Self::ViewState);
    fn update_view(view_state: &mut Self::ViewState, ucontext: &Context<Self>);
}

impl<C: Component> Comp<C>
where
    C: 'static + Component,
{
    pub fn context<'c>(&'c self, state: &'c C) -> Context<'c, C> {
        Context { comp: self, state }
    }

    pub fn callback<S>(&self, callback_fn: impl Fn(&mut C) -> S + 'static) -> CallbackFnArg<C, ()>
    where
        S: Into<ShouldRender>,
    {
        CallbackFnArg {
            comp: self.clone(),
            callback: Rc::new(move |state, _| callback_fn(state).into()),
        }
    }

    pub fn callback_arg<S, A>(
        &self,
        callback_fn: impl Fn(&mut C, A) -> S + 'static,
    ) -> CallbackArg<A>
    where
        A: 'static,
        S: Into<ShouldRender>,
    {
        Box::new(CallbackFnArg {
            comp: self.clone(),
            callback: Rc::new(move |state, arg| callback_fn(state, arg).into()),
        })
    }

    fn execute_given_callback_then_the_update_queue<A: 'static>(
        &self,
        arg: A,
        cb_fn: &CallbackFnArg<C, A>,
    ) {
        let need_to_execute_the_update_queue = update_queue_will_be_executing();

        self.execute_callback(arg, cb_fn);

        if need_to_execute_the_update_queue {
            execute_update_queue();
        }
    }

    fn execute_callback<A: 'static>(&self, arg: A, cb_fn: &CallbackFnArg<C, A>) {
        let Some(this) = self.0.upgrade() else {
            log::error!("Error on upgrading a WeakComp");
            return;
        };
        let Ok(mut comp_data) = this.try_borrow_mut() else {
            log::error!("Error on trying borrow mut a CompData");
            cb_fn.queue(arg);
            return;
        };
        let Some(comp_data) = comp_data.0.as_mut() else {
            log::error!("Error: No state, no updaters");
            cb_fn.queue(arg);
            return;
        };
        let should_render = (cb_fn.callback)(&mut comp_data.state, arg);
        if let ShouldRender::Yes = should_render {
            C::update_view(&mut comp_data.view_state, &self.context(&comp_data.state));
        }
    }
}

pub trait SpairSpawnLocal<T: 'static>: std::future::Future<Output = T> + Sized
where
    Self: 'static,
{
    fn spawn_local(self, callback: CallbackArg<T>) {
        let f = async move {
            let rs = self.await;
            callback.call(rs);
        };
        wasm_bindgen_futures::spawn_local(f);
    }
}

impl<T, F> SpairSpawnLocal<T> for F
where
    T: 'static,
    F: 'static + std::future::Future<Output = T>,
{
}
