use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    rc::{Rc, Weak},
};

use wasm_bindgen::UnwrapThrowExt;

use crate::{
    dom::{Element, WsElement},
    routing::{Route, get_current_location, setup_routing},
};

/// Start a component as an app without routing functionality.
pub fn start_app<C>(new_state: impl FnOnce(Comp<C>) -> C)
where
    C: Component + 'static,
{
    let rc_comp = create_component(new_state, |_, _: ()| {}, |_, _| {});
    std::mem::forget(rc_comp);
}

/// Start a component as an app with routing functionality.
pub fn start_app_with_routing<C, R>(new_state: impl FnOnce(Comp<C>) -> C, set_route: fn(&mut C, R))
where
    C: Component + 'static,
    R: Route + 'static,
{
    let rc_comp = create_component(new_state, set_route, setup_routing);
    std::mem::forget(rc_comp);
}

pub(crate) fn create_component<C, R>(
    new_state: impl FnOnce(Comp<C>) -> C,
    set_route: fn(&mut C, R),
    setup_routing: impl FnOnce(fn(&mut C, R), Comp<C>),
) -> RcComp<C>
where
    C: Component + 'static,
    R: Route,
{
    let comp_data = CompData {
        mounted: false,
        data: None,
    };
    let rc_comp = RcComp(Rc::new(RefCell::new(comp_data)));
    let mut state = new_state(rc_comp.comp());

    let route = R::from_location(&get_current_location());
    set_route(&mut state, route);
    setup_routing(set_route, rc_comp.comp());

    finalize_rc_comp(rc_comp, state)
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

pub trait Component: Sized {
    type ViewState: ComponentViewState;
    fn create(ccontext: &Context<Self>) -> Self::ViewState;
    fn update(view_state: &mut Self::ViewState, ucontext: &Context<Self>);
}

pub trait ComponentViewState {
    fn root_element(&self) -> &Element;
}

struct CompData<C>
where
    C: Component,
{
    mounted: bool,
    data: Option<CompDataInner<C>>,
}

struct CompDataInner<C>
where
    C: Component,
{
    root: WsElement,
    state: C,
    view_state: C::ViewState,
}

pub struct RcComp<C>(Rc<RefCell<CompData<C>>>)
where
    C: Component;
pub struct Comp<C>(Weak<RefCell<CompData<C>>>)
where
    C: Component;

pub struct Context<'a, C>
where
    C: Component,
{
    pub comp: &'a Comp<C>,
    pub state: &'a C,
}

impl<C> Clone for Comp<C>
where
    C: Component,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<C> Clone for Context<'_, C>
where
    C: Component,
{
    fn clone(&self) -> Self {
        Self {
            comp: self.comp,
            state: self.state,
        }
    }
}

impl<C> RcComp<C>
where
    C: Component + 'static,
{
    pub fn comp(&self) -> Comp<C> {
        Comp(Rc::downgrade(&self.0))
    }

    pub fn new(new_state: impl FnOnce(Comp<C>) -> C) -> Self {
        let comp_data = CompData {
            mounted: false,
            data: None,
        };
        let rc_comp = RcComp(Rc::new(RefCell::new(comp_data)));
        let comp = rc_comp.comp();
        finalize_rc_comp(rc_comp, new_state(comp))
    }

    pub fn root_element(&self) -> WsElement {
        self.0
            .try_borrow()
            .expect_throw("Error on borrowing RcComp content to get the component's root element")
            .data
            .as_ref()
            .expect_throw("RcComp CompData is empty")
            .root
            .clone()
    }

    /// Set `mounted` to the given value, return the old value
    pub fn set_mounted(&self, value: bool) -> bool {
        let mut comp_data = self
            .0
            .try_borrow_mut()
            .expect_throw("Error on borrowing RcComp content to set mounted");
        let old_value = comp_data.mounted;
        comp_data.mounted = value;
        old_value
    }

    pub fn create_comp_node(&self, parent: &WsElement, comp_marker: web_sys::Node) -> CompNode {
        let comp_node = CompNode {
            root_element: self.root_element(),
            comp_marker,
        };
        parent.insert_new_node_before_a_node(&comp_node.root_element, Some(&comp_node.comp_marker));
        self.set_mounted(true);
        comp_node
    }

    pub fn update_comp_node(&self, parent: &WsElement, comp_node: &mut CompNode) {
        let mounted = self.set_mounted(true);
        if mounted {
            return;
        }
        parent.remove_child(&comp_node.root_element);
        comp_node.root_element = self.root_element();
        parent.insert_new_node_before_a_node(&comp_node.root_element, Some(&comp_node.comp_marker));
    }
}

fn finalize_rc_comp<C>(rc_comp: RcComp<C>, state: C) -> RcComp<C>
where
    C: Component + 'static,
{
    let comp = rc_comp.comp();
    let context = comp.context(&state);
    let mut view_state = C::create(&context);
    C::update(&mut view_state, &context);

    match rc_comp.0.try_borrow_mut() {
        Ok(mut rc_comp) => {
            rc_comp.data = Some(CompDataInner {
                root: view_state.root_element().ws_element().clone(),
                state,
                view_state,
            });
        }
        _ => log::error!("Internal error: unable to mutable borrow rc_comp to set store its data"),
    }
    rc_comp
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

trait CallbackArgTrait<A> {
    fn execute(&self, arg: A);
}

pub struct CallbackArg<A>(Rc<dyn CallbackArgTrait<A>>);
pub struct Callback(CallbackArg<()>);

impl<A: 'static> CallbackArg<A> {
    pub fn clone(&self) -> Self {
        Self(self.0.clone())
    }

    pub fn call(&self, arg: A) {
        if is_update_queue_executing() {
            self.queue(arg);
            return;
        }
        self.0.execute(arg);
    }

    fn queue(&self, arg: A) {
        let clone = self.clone();
        put_callback_on_update_queue(move || clone.0.execute(arg));
    }
}

impl Callback {
    pub fn clone(&self) -> Self {
        Self(self.0.clone())
    }

    pub fn call(&self) {
        if is_update_queue_executing() {
            self.0.queue(());
            return;
        }
        self.0.0.execute(());
    }
}

type FnMutArg<C, A> = dyn Fn(&mut C, A) -> ShouldRender;
struct CallbackMutArgFn<C, A>
where
    C: Component,
{
    comp: Comp<C>,
    callback: Box<FnMutArg<C, A>>,
}

impl<C, A> CallbackArgTrait<A> for CallbackMutArgFn<C, A>
where
    C: 'static + Component,
    A: 'static,
{
    fn execute(&self, arg: A) {
        self.comp
            .execute_given_callback_then_the_update_queue(arg, self);
    }
}

impl<C: Component> Comp<C>
where
    C: 'static + Component,
{
    pub fn context<'c>(&'c self, state: &'c C) -> Context<'c, C> {
        Context { comp: self, state }
    }

    pub fn callback<S>(&self, callback_fn: impl Fn(&mut C) -> S + 'static) -> Callback
    where
        S: Into<ShouldRender>,
    {
        let cba = CallbackMutArgFn {
            comp: self.clone(),
            callback: Box::new(move |state, _| callback_fn(state).into()),
        };
        Callback(CallbackArg(Rc::new(cba)))
    }

    pub fn callback_arg<S, A>(
        &self,
        callback_fn: impl Fn(&mut C, A) -> S + 'static,
    ) -> CallbackArg<A>
    where
        S: Into<ShouldRender>,
        A: 'static,
    {
        let cba = CallbackMutArgFn {
            comp: self.clone(),
            callback: Box::new(move |state, arg| callback_fn(state, arg).into()),
        };
        CallbackArg(Rc::new(cba))
    }

    fn execute_given_callback_then_the_update_queue<A: 'static>(
        &self,
        arg: A,
        cb_fn: &CallbackMutArgFn<C, A>,
    ) {
        let need_to_execute_the_update_queue = update_queue_will_be_executing();

        self.execute_callback(arg, cb_fn);

        if need_to_execute_the_update_queue {
            execute_update_queue();
        }
    }

    fn execute_callback<A: 'static>(&self, arg: A, cb_fn: &CallbackMutArgFn<C, A>) {
        let Some(this) = self.0.upgrade() else {
            log::error!(
                "Error on upgrading a WeakComp. A callback-executing has been interupted and discarded"
            );
            // Does the component has been disposed?
            return;
        };
        let Ok(mut comp_data) = this.try_borrow_mut() else {
            log::error!(
                "Error on trying borrow mut a CompData. A callback-executing has been interupted and discarded"
            );
            // cb_fn.queue(arg); To queue again, CallbackMutArgFn::callback must be changed to Rc
            return;
        };
        let Some(comp_data) = comp_data.data.as_mut() else {
            log::error!("Error: No state, no updaters");
            // cb_fn.queue(arg); // same as above
            return;
        };
        let should_render = (cb_fn.callback)(&mut comp_data.state, arg);
        if let ShouldRender::Yes = should_render {
            C::update(&mut comp_data.view_state, &self.context(&comp_data.state));
        }
    }
}

pub trait SpairSpawnLocal: std::future::Future<Output = ()> + Sized
where
    Self: 'static,
{
    fn spawn_local(self) {
        wasm_bindgen_futures::spawn_local(self);
    }
}

impl<F> SpairSpawnLocal for F where F: 'static + std::future::Future<Output = ()> {}

pub trait SpairSpawnLocalWithCallback<T: 'static>: std::future::Future<Output = T> + Sized
where
    Self: 'static,
{
    fn spawn_local_with_callback(self, callback: CallbackArg<T>) {
        let f = async move {
            let rs = self.await;
            callback.call(rs);
        };
        wasm_bindgen_futures::spawn_local(f);
    }
}

impl<T, F> SpairSpawnLocalWithCallback<T> for F
where
    T: 'static,
    F: 'static + std::future::Future<Output = T>,
{
}

#[doc(hidden)]
pub struct CompNode {
    pub root_element: WsElement,
    pub comp_marker: web_sys::Node,
}
