use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::future::Future;
use std::rc::{Rc, Weak};
use wasm_bindgen::UnwrapThrowExt;

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

fn i_have_to_execute_update_queue() -> bool {
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

pub fn update_component(fn_update: impl FnOnce() + 'static) {
    UPDATE_QUEUE.with(|uq| uq.add(Box::new(fn_update)));
}

fn execute_update_queue(promise: bool) {
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
    type Routes: crate::routing::Routes<Self>;
    // It return Option to allow default implementation
    fn with_comp(_: Comp<Self>) -> Option<Self> {
        None
    }

    // Better name?
    // This method will be ran once when the component is created.
    fn initialize(_: &Comp<Self>) {}

    fn default_checklist() -> Checklist<Self> {
        Self::default_should_render().into()
    }

    fn default_should_render() -> ShouldRender {
        ShouldRender::Yes
    }

    fn render<'a>(&self, element: crate::Element<'a, Self>);
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

pub struct RcComp<C>(Rc<RefCell<CompInstance<C>>>);
pub struct Comp<C>(Weak<RefCell<CompInstance<C>>>);

pub struct CompInstance<C> {
    state: Option<C>,
    root_element: crate::dom::Element,
    router: Option<crate::routing::Router>,
    mount_status: MountStatus,
    events: Vec<Box<dyn crate::events::Listener>>,
}

pub enum MountStatus {
    // This is for a child component, when it was created but not mount yet.
    Never,
    // A child component that is attached to the DOM.
    Mounted,
    // A child component that is previously attached to the DOM but
    // has been detached.
    Unmounted,
    // The main component always in this status.
    PermanentlyMounted,
}

#[must_use = "This value must be returned to the framework. Otherwise, it will be lost and the default value will be used"]
pub struct Checklist<C: Component> {
    should_render: ShouldRender,
    commands: Commands<C>,
}

struct Commands<C>(Vec<Box<dyn Command<C>>>);

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
    fn into_parts(self) -> (ShouldRender, Commands<C>) {
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

impl<C> RcComp<C> {
    pub(crate) fn new(root: Option<web_sys::Element>) -> Self {
        let (root_element, mount_status) = root
            .map(|root| {
                let root = crate::dom::Element::from_ws_element(root);
                (root, MountStatus::PermanentlyMounted)
            })
            .unwrap_or_else(|| {
                // Just an element to create CompInstance, the element will be replace by the
                // actual node when attaching to the DOM
                (crate::dom::Element::new("div"), MountStatus::Never)
            });

        Self(Rc::new(RefCell::new(CompInstance {
            state: None,
            root_element,
            router: None,
            mount_status,
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
        use crate::routing::Routes;
        let comp = self.comp();

        C::initialize(&comp);

        let promise = self::i_have_to_execute_update_queue();

        // The router may cause an update that mutably borrows the CompInstance
        // Therefor this should be done before borrowing the instance.
        let router = C::Routes::router(&comp);

        {
            let mut instance = self
                .0
                .try_borrow_mut()
                .expect_throw("Expect no borrowing at the first render");

            if instance.root_element.is_empty() {
                // In cases that the router not cause any render yet, such as Routes = ()
                instance.render(&comp);
            }

            instance.router = router;
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

    fn set_mount_status_to_unmounted(&self) {
        if let Some(instance) = self.0.upgrade() {
            if let Ok(mut instance) = instance.try_borrow_mut() {
                instance.mount_status = MountStatus::Unmounted;
            }
        }
    }

    fn update<Cl>(&self, fn_update: &Rc<impl Fn(&mut C) -> Cl + 'static>)
    where
        Cl: Into<Checklist<C>>,
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
                    let comp = self.clone();
                    let fn_update = Rc::clone(fn_update);
                    self::update_component(move || comp.update(&fn_update));
                    return;
                }
            };

            // Call `fn_update` here to reduce monomorphization on `CompInstance::extra_update()`
            // Otherwise, `extra_update` need another type parameter `fn_update: &impl Fn(&mut C) -> Cl`.
            let (skip_fn_render, commands) = fn_update(this.state.as_mut().unwrap_throw())
                .into()
                .into_parts();
            this.extra_update(skip_fn_render, commands, &self);
        }
        self::execute_update_queue(promise);
    }

    fn update_arg<T: 'static, Cl>(&self, arg: T, fn_update: &Rc<impl Fn(&mut C, T) -> Cl + 'static>)
    where
        Cl: Into<Checklist<C>>,
    {
        let promise = self::i_have_to_execute_update_queue();
        {
            let this = self
                .0
                .upgrade()
                .expect_throw("Expect the component instance alive when updating - update_arg()");
            let mut this = match this.try_borrow_mut() {
                Ok(this) => this,
                Err(_) => {
                    let comp = self.clone();
                    let fn_update = Rc::clone(fn_update);
                    self::update_component(move || comp.update_arg(arg, &fn_update));
                    return;
                }
            };

            // Call `fn_update` here to reduce monomorphization on `CompInstance::extra_update()`
            // Otherwise, `extra_update` need another type parameter `fn_update: &impl Fn(&mut C) -> Cl`.
            let (skip_fn_render, commands) = fn_update(this.state.as_mut().unwrap_throw(), arg)
                .into()
                .into_parts();
            this.extra_update(skip_fn_render, commands, &self);
        }
        self::execute_update_queue(promise);
    }

    pub fn callback<Cl>(&self, fn_update: impl Fn(&mut C) -> Cl + 'static) -> impl Fn()
    where
        Cl: Into<Checklist<C>>,
    {
        let comp = self.clone();
        let fn_update = Rc::new(fn_update);
        move || comp.update(&fn_update)
    }

    pub fn callback_arg<T: 'static, Cl>(
        &self,
        fn_update: impl Fn(&mut C, T) -> Cl + 'static,
    ) -> impl Fn(T)
    where
        Cl: Into<Checklist<C>>,
    {
        let comp = self.clone();
        let fn_update = Rc::new(fn_update);
        move |t: T| comp.update_arg(t, &fn_update)
    }

    pub fn handler<T: 'static, Cl>(&self, fn_update: impl Fn(&mut C) -> Cl + 'static) -> impl Fn(T)
    where
        Cl: Into<Checklist<C>>,
    {
        let comp = self.clone();
        let fn_update = Rc::new(fn_update);
        move |_: T| comp.update(&fn_update)
    }

    pub fn handler_arg<T: 'static, Cl>(
        &self,
        fn_update: impl Fn(&mut C, T) -> Cl + 'static,
    ) -> impl Fn(T)
    where
        Cl: Into<Checklist<C>>,
    {
        self.callback_arg(fn_update)
    }

    pub fn async_handler<T: 'static, R: 'static, Cl, F>(
        &self,
        fn_update: impl Fn(&mut C, Result<R, wasm_bindgen::JsValue>) -> Cl + 'static,
        future_creator: impl Fn() -> F + 'static,
    ) -> impl Fn(T)
    where
        Cl: Into<Checklist<C>>,
        F: 'static + Future<Output = Result<R, wasm_bindgen::JsValue>>,
    {
        let comp = self.clone();
        let fn_update = Rc::new(fn_update);
        let future_creator = Rc::new(future_creator);

        move |_: T| {
            let comp = comp.clone();
            let fn_update = fn_update.clone();
            let future_creator = future_creator.clone();
            let f = async move {
                let output = future_creator().await;
                comp.update_arg(output, &fn_update);
            };
            wasm_bindgen_futures::spawn_local(f);
        }
    }
}

impl<C: Component> CompInstance<C> {
    pub(crate) fn render(&mut self, comp: &Comp<C>) {
        let state = self.state.as_ref().unwrap_throw();
        let status = if self.root_element.is_empty() {
            crate::dom::ElementStatus::JustCreated
        } else {
            crate::dom::ElementStatus::Existing
        };
        let eu = crate::dom::ElementUpdater {
            comp_context: crate::dom::CompContext { state, comp },
            el_context: self.root_element.create_context(status),
        };
        state.render(eu);
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
        commands.execute(comp, self.state.as_mut().unwrap_throw());
    }

    pub fn state(&self) -> &C {
        self.state.as_ref().unwrap_throw()
    }

    pub(crate) fn is_mounted(&self) -> bool {
        match self.mount_status {
            MountStatus::Mounted => true,
            _ => false,
        }
    }
}

pub type ChildComp<C> = RcComp<C>;

impl<C: Component> ChildComp<C> {
    // Attach the component to the given ws_element, and run the render
    pub(crate) fn mount_to(&self, ws_element: &web_sys::Element) {
        let comp = self.comp();

        C::initialize(&comp);

        let promise = self::i_have_to_execute_update_queue();

        {
            let mut instance = self
                .0
                .try_borrow_mut()
                .expect_throw("Why unable to borrow a child component on attaching?");

            // TODO: This may cause problems
            //  * When the component was detached from an element then
            //      was attached to another element with mismatched attributes?
            //  * When the component was detached and reattached to the
            //      same element but somehow attributes are still mismatched?
            //      because there is another component was attached in between?
            instance.root_element.replace_ws_element(ws_element.clone());

            instance.mount_status = MountStatus::Mounted;

            // TODO: Allow an option to ignore render on re-mounted?
            instance.render(&comp);
        }
        self::execute_update_queue(promise);
    }

    pub fn comp_instance(&self) -> std::cell::Ref<CompInstance<C>> {
        self.0.borrow()
    }
}

impl<C: Component> From<C> for ChildComp<C> {
    fn from(state: C) -> Self {
        let rc_comp = ChildComp::new(None);
        rc_comp.set_state(state);
        rc_comp
    }
}

pub trait WithParentComp: Sized {
    type Parent;
    fn with_parent_and_comp(parent: &Comp<Self::Parent>, comp: Comp<Self>) -> Self;
}

impl<C: WithParentComp + Component> ChildComp<C> {
    pub fn with_parent(parent: &Comp<C::Parent>) -> Self {
        let rc_comp = ChildComp::new(None);
        rc_comp.set_state(C::with_parent_and_comp(parent, rc_comp.comp()));
        rc_comp
    }
}

// A new struct and impl Drop on it, instead of impl Drop on Comp,
// because we only want to set status to unmounted when removing
// it from its parent.
pub struct ComponentHandle<C: Component>(Comp<C>);

impl<C: Component> Drop for ComponentHandle<C> {
    fn drop(&mut self) {
        self.0.set_mount_status_to_unmounted();
    }
}

impl<C: Component> From<Comp<C>> for ComponentHandle<C> {
    fn from(comp: Comp<C>) -> Self {
        Self(comp)
    }
}

impl<C> Drop for ChildComp<C> {
    fn drop(&mut self) {
        self.0
            .try_borrow_mut()
            .expect_throw("Why unable to borrow a child component in dropping?")
            .root_element
            .ws_element()
            .set_text_content(None);
    }
}
