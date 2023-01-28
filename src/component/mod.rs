use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::rc::{Rc, Weak};
use wasm_bindgen::UnwrapThrowExt;

use crate::dom::{Element, ElementStatus};

mod child_component;

pub use child_component::*;

struct UpdateQueue {
    // A hack to avoid circular borrowing and mutable borrowing between
    // child and parent component. The first component that run when
    // `will_be_executed` == false will have to promise that it will
    // execute UPDATE_QUEUE.
    will_be_executed: Cell<bool>,
    queue: RefCell<VecDeque<Box<dyn FnOnce()>>>,
}

thread_local! {
    static UPDATE_QUEUE: UpdateQueue = UpdateQueue {
        will_be_executed: Cell::new(false),
        queue: RefCell::new(VecDeque::new())
    };
}

pub fn i_have_to_execute_update_queue() -> bool {
    UPDATE_QUEUE.with(|uq| {
        match uq.will_be_executed.get() {
            false => {
                // It means that there is no component promise executing UPDATE_QUEUE
                // The caller must execute it.
                uq.will_be_executed.set(true);
                true
            }
            true => {
                // There already a component promise to execute UPDATE_QUEUE, so the
                // caller just ignore it.
                false
            }
        }
    })
}

pub fn update_queue_will_be_execute() -> bool {
    UPDATE_QUEUE.with(|uq| uq.will_be_executed.get())
}

pub fn update_component(fn_update: impl FnOnce() + 'static) {
    UPDATE_QUEUE.with(|uq| uq.add(Box::new(fn_update)));
}

pub fn execute_update_queue(promise: bool) {
    if !promise {
        return;
    }
    UPDATE_QUEUE.with(|uq| uq.execute());
}

impl UpdateQueue {
    fn add(&self, f: Box<dyn FnOnce()>) {
        self.queue.borrow_mut().push_back(f);
    }

    fn take(&self) -> Option<Box<dyn FnOnce()>> {
        self.queue.borrow_mut().pop_front()
    }

    fn execute(&self) {
        while let Some(f) = self.take() {
            f();
        }
        self.will_be_executed.set(false);
    }
}

pub trait Component: 'static + Sized {
    type Routes: crate::routing::Routes;

    fn debug(&self) -> &str {
        "Default Component::debug()"
    }

    // This method will be ran once when the component is created.
    fn init(_: &Comp<Self>) {}

    /// This method allow child-components that wants to receive update when the location
    /// changes to register its callback to the router. The root-component (the component that
    /// implements `spair::Application`) does not have to implement this, but must implement
    /// `spair::Application::init_router`. This method will never be called on the root-component.
    fn register_routing_callback(
        _router: &mut <Self::Routes as crate::routing::Routes>::Router,
        _comp: &Comp<Self>,
    ) {
    }

    fn remove_routing_callback(_router: &mut <Self::Routes as crate::routing::Routes>::Router) {}

    fn default_checklist() -> Checklist<Self> {
        Self::default_should_render().into()
    }

    fn default_should_render() -> ShouldRender {
        ShouldRender::Yes
    }

    /// This method will be called before executing an update method
    fn before_update(&mut self) {}

    fn render(&self, element: crate::Element<Self>);
}

#[must_use = "This value must be returned to the framework. Otherwise, it will be lost and the default value will be used"]
pub enum ShouldRender {
    No,
    Yes,
}

impl<C: Component> From<ShouldRender> for Checklist<C> {
    fn from(should_render: ShouldRender) -> Self {
        Checklist {
            should_render,
            commands: Commands(Vec::new()),
        }
    }
}

pub struct RcComp<C: Component>(Rc<RefCell<CompInstance<C>>>);
pub struct Comp<C: Component>(Weak<RefCell<CompInstance<C>>>);

pub struct CompInstance<C> {
    state: Option<C>,
    root_element: Element,
    mount_status: ComponentMountStatus,
    events: Vec<Box<dyn crate::events::Listener>>,
}

#[derive(Debug)]
pub enum ComponentMountStatus {
    // A child component that is attached to the DOM.
    Mounted,
    // A child component that is not attached from the DOM.
    Unmounted,
    // The main component always in this status.
    PermanentlyMounted,
}

#[must_use = "This value must be returned to the framework. Otherwise, it will be lost and the default value will be used"]
pub struct Checklist<C: Component> {
    should_render: ShouldRender,
    commands: Commands<C>,
}

pub(crate) struct Commands<C>(Vec<Box<dyn Command<C>>>);

impl<C: Component> Commands<C> {
    fn execute(&mut self, comp: &Comp<C>, state: &mut C) {
        self.0.iter_mut().for_each(|c| c.execute(comp, state));
    }
}

pub trait Command<C: Component> {
    fn execute(&mut self, comp: &Comp<C>, state: &mut C);
}

impl<C: Component> From<()> for Checklist<C> {
    fn from(_: ()) -> Self {
        C::default_checklist()
    }
}

impl<C: Component> Checklist<C> {
    pub(crate) fn into_parts(self) -> (ShouldRender, Commands<C>) {
        (self.should_render, self.commands)
    }

    pub fn should_render() -> Self {
        Self {
            should_render: ShouldRender::Yes,
            commands: Commands(Vec::new()),
        }
    }

    pub fn skip_render() -> Self {
        Self {
            should_render: ShouldRender::No,
            commands: Commands(Vec::new()),
        }
    }

    pub fn set_should_render(&mut self) {
        self.should_render = ShouldRender::Yes;
    }

    pub fn set_skip_render(&mut self) {
        self.should_render = ShouldRender::No;
    }

    pub fn add_option_command(&mut self, cmd: crate::OptionCommand<C>) {
        if let Some(cmd) = cmd.0 {
            self.commands.0.push(cmd);
        }
    }

    pub fn add_command(&mut self, cmd: crate::Command<C>) {
        self.commands.0.push(cmd.0);
    }
}

impl<C: Component> RcComp<C> {
    // This method should only be used to construct the root component of the application,
    // which is the component that impled crate::application::Application
    pub(crate) fn with_ws_root(root: web_sys::Element) -> Self {
        let root_element = Element::from_ws_element(root);
        let mount_status = ComponentMountStatus::PermanentlyMounted;

        Self(Rc::new(RefCell::new(CompInstance {
            state: None,
            root_element,
            mount_status,
            events: Vec::new(),
        })))
    }

    pub(crate) fn with_root(root_element: Element) -> Self {
        Self(Rc::new(RefCell::new(CompInstance {
            state: None,
            root_element,
            mount_status: ComponentMountStatus::Unmounted,
            events: Vec::new(),
        })))
    }
}

impl<C: Component> RcComp<C> {
    pub(crate) fn set_state(&self, state: C) {
        self.0
            .try_borrow_mut()
            .expect_throw("Why unable to mutably borrow comp instance to set state?")
            .state = Some(state);
    }

    pub(crate) fn first_render(&self) {
        let comp = self.comp();

        C::init(&comp);

        let promise = self::i_have_to_execute_update_queue();

        {
            // This borrow_mut must end before executing self::execute_update_queue()
            let mut instance = self
                .0
                .try_borrow_mut()
                .expect_throw("Expect no borrowing at the first render");

            if instance.root_element.is_empty() {
                // In cases that the router not cause any render yet, such as Routes = ()
                instance.render(&comp);
            }
        }

        self::execute_update_queue(promise);
    }

    pub fn comp(&self) -> Comp<C> {
        Comp(Rc::downgrade(&self.0))
    }
}

impl<C: Component> Clone for Comp<C> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<C: Component> Comp<C> {
    pub fn window_event(&self, listener: Box<dyn crate::events::Listener>) -> &Self {
        self.0
            .upgrade()
            .expect_throw("Comp::window_event: why the component dropped?")
            .try_borrow_mut()
            .expect_throw("Why unable to mutably borrow comp instance to store event?")
            .events
            .push(listener);
        self
    }

    #[cfg(feature = "queue-render")]
    pub(crate) fn upgrade(&self) -> Rc<RefCell<CompInstance<C>>> {
        // Why wrapping this around an RcComp cause a bug the clear the root element empty?
        self.0
            .upgrade()
            .expect_throw("Comp::upgrade: why the component dropped?")
    }

    fn set_mount_status_to_unmounted(&self) {
        if let Some(instance) = self.0.upgrade() {
            if let Ok(mut instance) = instance.try_borrow_mut() {
                instance.mount_status = ComponentMountStatus::Unmounted;
            }
        }
    }

    pub(crate) fn execute_callback<A, Cb>(&self, arg: A, callback: Cb)
    where
        Cb: crate::callback::ExecuteCallback<C, A>,
    {
        let promise = self::i_have_to_execute_update_queue();
        {
            let this = self
                .0
                .upgrade()
                .expect_throw("Expect the component instance alive when updating - update()");
            let mut this = match this.try_borrow_mut() {
                Ok(this) => this,
                Err(_) => {
                    callback.queue(arg);
                    return;
                }
            };

            let state = this
                .state
                .as_mut()
                .expect_throw("Mutable reference to state for updating");
            C::before_update(state);
            let (should_render, commands) = callback.execute(state, arg).into_parts();
            this.extra_update(should_render, commands, self);
        }
        self::execute_update_queue(promise);
        #[cfg(feature = "queue-render")]
        crate::queue_render::execute_render_queue();
    }

    pub fn callback_once_mut<Cl, F>(&self, f: F) -> crate::callback::CallbackFnOnce<C, Cl, F>
    where
        Cl: 'static + Into<Checklist<C>>,
        F: 'static + FnOnce(&mut C) -> Cl,
    {
        crate::callback::CallbackFnOnce {
            comp: self.clone(),
            callback: f,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn callback_once_arg_mut<Cl, A, F>(
        &self,
        f: F,
    ) -> crate::callback::CallbackFnOnceArg<C, A, Cl, F>
    where
        Cl: 'static + Into<Checklist<C>>,
        F: 'static + FnOnce(&mut C, A) -> Cl,
    {
        crate::callback::CallbackFnOnceArg {
            comp: self.clone(),
            callback: f,
            _phantom: std::marker::PhantomData,
        }
    }

    fn cb<Cl: 'static>(&self, f: impl Fn(&C) -> Cl + 'static) -> crate::callback::CallbackFn<C, ()>
    where
        Cl: Into<Checklist<C>>,
    {
        crate::callback::CallbackFn {
            comp: self.clone(),
            callback: Rc::new(crate::callback::Cb(f)),
        }
    }

    fn cb_mut<Cl: 'static>(
        &self,
        f: impl Fn(&mut C) -> Cl + 'static,
    ) -> crate::callback::CallbackFn<C, ()>
    where
        Cl: Into<Checklist<C>>,
    {
        crate::callback::CallbackFn {
            comp: self.clone(),
            callback: Rc::new(crate::callback::CbMut(f)),
        }
    }

    fn cb_dropped_arg<Cl: 'static, A>(
        &self,
        f: impl Fn(&C) -> Cl + 'static,
    ) -> crate::callback::CallbackFn<C, A>
    where
        Cl: Into<Checklist<C>>,
    {
        crate::callback::CallbackFn {
            comp: self.clone(),
            callback: Rc::new(crate::callback::CbDroppedArg(f)),
        }
    }

    fn cb_dropped_arg_mut<Cl: 'static, A>(
        &self,
        f: impl Fn(&mut C) -> Cl + 'static,
    ) -> crate::callback::CallbackFn<C, A>
    where
        Cl: Into<Checklist<C>>,
    {
        crate::callback::CallbackFn {
            comp: self.clone(),
            callback: Rc::new(crate::callback::CbDroppedArgMut(f)),
        }
    }

    fn cb_arg_mut<Cl: 'static, A>(
        &self,
        f: impl Fn(&mut C, A) -> Cl + 'static,
    ) -> crate::callback::CallbackFn<C, A>
    where
        Cl: Into<Checklist<C>>,
    {
        crate::callback::CallbackFn {
            comp: self.clone(),
            callback: Rc::new(crate::callback::CbArgMut(f)),
        }
    }

    pub fn callback<Cl: 'static>(&self, f: impl Fn(&C) -> Cl + 'static) -> crate::Callback
    where
        Cl: Into<Checklist<C>>,
    {
        Box::new(self.cb(f))
    }

    pub fn callback_mut<Cl: 'static>(&self, f: impl Fn(&mut C) -> Cl + 'static) -> crate::Callback
    where
        Cl: Into<Checklist<C>>,
    {
        Box::new(self.cb_mut(f))
    }

    pub fn callback_arg_mut<Cl: 'static, A: 'static>(
        &self,
        f: impl Fn(&mut C, A) -> Cl + 'static,
    ) -> crate::CallbackArg<A>
    where
        Cl: Into<Checklist<C>>,
    {
        Box::new(self.cb_arg_mut(f))
    }

    pub fn handler<Cl: 'static, A: 'static>(
        &self,
        f: impl Fn(&C) -> Cl + 'static,
    ) -> impl crate::callback::CallbackArg<A>
    where
        Cl: Into<Checklist<C>>,
    {
        self.cb_dropped_arg(f)
    }

    pub fn handler_mut<Cl: 'static, A: 'static>(
        &self,
        f: impl Fn(&mut C) -> Cl + 'static,
    ) -> impl crate::callback::CallbackArg<A>
    where
        Cl: Into<Checklist<C>>,
    {
        self.cb_dropped_arg_mut(f)
    }

    pub fn handler_arg_mut<Cl: 'static, A: 'static>(
        &self,
        f: impl Fn(&mut C, A) -> Cl + 'static,
    ) -> impl crate::callback::CallbackArg<A>
    where
        Cl: Into<Checklist<C>>,
    {
        self.cb_arg_mut(f)
    }
}

impl<C: Component> CompInstance<C> {
    pub(crate) fn render(&mut self, comp: &Comp<C>) {
        let state = self
            .state
            .as_ref()
            .expect_throw("A immutable reference for rendering component");
        let status = if self.root_element.is_empty() {
            ElementStatus::JustCreated
        } else {
            ElementStatus::Existing
        };
        let er =
            crate::render::base::ElementUpdater::new(comp, state, &mut self.root_element, status);
        state.render(er.into());
    }

    fn extra_update(
        &mut self,
        should_render: ShouldRender,
        mut commands: Commands<C>,
        comp: &Comp<C>,
    ) {
        if let ShouldRender::Yes = should_render {
            self.render(comp);
        }
        commands.execute(
            comp,
            self.state
                .as_mut()
                .expect_throw("A mutable reference for executing commands"),
        );
    }

    pub fn state(&self) -> &C {
        self.state
            .as_ref()
            .expect_throw("Immutably borrow the state from CompInstance::state()")
    }

    pub(crate) fn is_mounted(&self) -> bool {
        matches!(self.mount_status, ComponentMountStatus::Mounted)
    }

}
