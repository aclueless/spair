use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    rc::{Rc, Weak},
};

use crate::elements::WsElement;

// pub fn mount_to_body<C: Component>(state: C) {
//     let Some(window) = web_sys::window() else {
//         log::error!("No window found. Can not mount to document body");
//         return;
//     };
//     let Some(document) = window.document() else {
//         log::error!("No document found. Can not mount to document body");
//         return;
//     };
//     let Some(body) = document.body() else {
//         log::error!("No body found. Can not mount to document body");
//         return;
//     };

//     mount_to(state, body.into())
// }

// pub fn mount_to<C: Component>(state: C, root: web_sys::Element) {
//     let comp_data = CompData {
//         root: Element::new(root, 0),
//         state_n_updaters: None,
//     };
//     let rc_comp = Rc::new(RefCell::new(comp_data));
//     let comp = Comp(Rc::downgrade(&rc_comp));
//     let (root, mut updater) = C::init(&comp);

//     if let Ok(mut rc_comp) = rc_comp.try_borrow_mut() {
//         root.merge_to(&mut rc_comp.root);
//         state.render(&mut updater, &comp);
//         rc_comp.state_n_updaters = Some((state, updater));
//     }

//     std::mem::forget(rc_comp);
// }

pub fn start_app<C: Component>(state: C) {
    let rc_comp = create_component(state);
    std::mem::forget(rc_comp);
}

pub fn create_component<C: Component>(state: C) -> RcComp<C> {
    let comp_data = CompData(None);
    let rc_comp = Rc::new(RefCell::new(comp_data));
    let comp = Comp(Rc::downgrade(&rc_comp));
    let (root, updaters) = state.init(&comp);

    match rc_comp.try_borrow_mut() {
        Ok(mut rc_comp) => {
            // state.render(&mut updaters, &comp);
            rc_comp.0 = Some(CompDataInner {
                _root: root,
                state,
                updaters,
            });
        }
        _ => log::error!("Internal error: unable to mutable borrow rc_comp to set store its data"),
    }
    RcComp(rc_comp)
}

thread_local! {
    static UPDATE_QUEUE_IS_IN_EXECUTING: Cell<bool> = const { Cell::new(false) };
    static UPDATE_QUEUE: RefCell<VecDeque<Box<dyn FnOnce()>>> = RefCell::new(VecDeque::new());
}

fn is_update_queue_executing() -> bool {
    UPDATE_QUEUE_IS_IN_EXECUTING.with(|executing| executing.replace(true))
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
    _root: ComponentRoot,
    state: C,
    updaters: C::ViewState,
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

#[allow(dead_code)]
impl<C: Component> RcComp<C> {
    pub fn comp(&self) -> Comp<C> {
        Comp(Rc::downgrade(&self.0))
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

    pub fn execute(&self, arg: A) {
        self.comp.execute_or_queue_callback(arg, self);
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
        self.execute(arg)
    }
}

pub trait Component: Sized {
    type ViewState;
    fn init(&self, comp: &Comp<Self>) -> (ComponentRoot, Self::ViewState);
    fn render(&self, updater: &mut Self::ViewState, comp: &Comp<Self>);
}

pub enum ComponentRoot {
    Body,
    Element(WsElement),
}

impl<C: Component> Comp<C>
where
    C: 'static + Component,
{
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

    fn execute_or_queue_callback<A: 'static>(&self, arg: A, cb_fn: &CallbackFnArg<C, A>) {
        let executing = is_update_queue_executing();
        if executing {
            cb_fn.queue(arg);
            return;
        }

        self.execute_callback(arg, cb_fn);

        if !executing {
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
            comp_data.state.render(&mut comp_data.updaters, self);
        }
    }
}
