use std::cell::RefCell;
use std::rc::{Rc, Weak};
use wasm_bindgen::UnwrapThrowExt;

pub trait Component: 'static + Sized {
    type Routes: crate::routing::Routes<Self>;
    type Components: Components<Self>;
    // fn init() -> Self;
    fn render<'a>(&self, context: Context<'a, Self>);
}

pub struct Context<'a, C: Component> {
    pub element: crate::dom::ElementUpdater<'a, C>,
    pub comp: &'a Comp<C>,
    pub child_components: &'a C::Components,
}

impl<'a, C: Component> Context<'a, C> {
    pub fn new(
        comp: &'a Comp<C>,
        element: crate::dom::ElementUpdater<'a, C>,
        child_components: &'a C::Components,
    ) -> Self {
        Self {
            comp,
            element,
            child_components,
        }
    }

    pub fn into_comp_element(self) -> (&'a Comp<C>, crate::dom::ElementUpdater<'a, C>) {
        (self.comp, self.element)
    }

    pub fn into_parts(
        self,
    ) -> (
        &'a Comp<C>,
        crate::dom::ElementUpdater<'a, C>,
        &'a C::Components,
    ) {
        (self.comp, self.element, self.child_components)
    }
}

pub struct RcComp<C: Component>(Rc<RefCell<CompInstance<C>>>);
pub struct Comp<C: Component>(Weak<RefCell<CompInstance<C>>>);

pub struct CompInstance<C: Component> {
    state: C,
    child_components: Option<C::Components>,
    root_element: crate::dom::Element,
    router: Option<crate::routing::Router>,
    mount_status: MountStatus,
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
    AlwaysMounted,
}

#[must_use]
pub struct Checklist<C: Component> {
    skip_fn_render: bool,
    commands: Commands<C>,
    related_comp_updates: RelatedCompUpdates,
}

struct Commands<C>(Vec<Box<dyn Command<C>>>);
struct RelatedCompUpdates(Vec<Box<dyn Fn()>>);

impl<C: Component> Commands<C> {
    fn execute(&mut self, comp: &Comp<C>, state: &mut C) {
        self.0.iter_mut().for_each(|c| c.execute(comp, state));
    }
}

impl RelatedCompUpdates {
    fn execute(&self) {
        self.0.iter().for_each(|c| c());
    }
}

pub trait Command<C: Component> {
    fn execute(&mut self, comp: &Comp<C>, state: &mut C);
}

impl<C: Component> Default for Checklist<C> {
    fn default() -> Self {
        Self {
            skip_fn_render: false,
            commands: Commands(Vec::new()),
            related_comp_updates: RelatedCompUpdates(Vec::new()),
        }
    }
}

impl<C: Component> From<()> for Checklist<C> {
    fn from(_: ()) -> Self {
        Self::default()
    }
}

impl<C: Component> Checklist<C> {
    fn into_parts(self) -> (bool, Commands<C>, RelatedCompUpdates) {
        (
            self.skip_fn_render,
            self.commands,
            self.related_comp_updates,
        )
    }

    pub fn skip_fn_render() -> Self {
        let mut s = Self::default();
        s.skip_fn_render = true;
        s
    }

    pub fn fetch_json_ok_error<R, Cl>(
        &mut self,
        req: http::Request<Option<String>>,
        options: Option<crate::fetch::FetchOptions>,
        ok: fn(&mut C, R) -> Cl,
        error: fn(&mut C, crate::FetchError),
    ) where
        R: 'static + serde::de::DeserializeOwned,
        Cl: 'static + Into<Checklist<C>>,
    {
        self.commands
            .0
            .push(Box::new(crate::fetch::FetchCommand::new(
                req, options, ok, error,
            )));
    }

    pub fn update_related_component(&mut self, fn_update: impl Fn() + 'static) {
        self.related_comp_updates.0.push(Box::new(fn_update));
    }
}

impl<C: Component> RcComp<C> {
    pub fn with_state_and_element(state: C, root: Option<web_sys::Element>) -> Self {
        let (root_element, mount_status) = root
            .map(|root| {
                (
                    crate::dom::Element::from_ws_element(root),
                    MountStatus::AlwaysMounted,
                )
            })
            .unwrap_or_else(|| {
                // Just an element to create CompInstance, the element will be replace by the
                // actual node when attaching to the DOM
                (
                    crate::dom::Element::new("div"),
                    // crate::utils::document()
                    //     .create_element("div")
                    //     .expect_throw("Unable to create a div to use as a phantom node"),
                    MountStatus::Never,
                )
            });

        let rc = Self(Rc::new(RefCell::new(CompInstance {
            state,
            root_element,
            router: None,
            child_components: None,
            mount_status,
        })));
        {
            let comp = rc.comp();
            let mut instance = rc.0.try_borrow_mut().unwrap_throw();
            instance.child_components = Some(C::Components::new(&instance.state, comp));
        }
        rc
    }

    pub fn first_render(&self) {
        use crate::routing::Routes;
        let comp = self.comp();

        // The router may cause an update that mutably borrows the CompInstance
        // Therefor this should be done before borrowing the instance.
        let router = C::Routes::router(&comp);

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
    fn set_mount_status_to_unmounted(&self) {
        if let Some(instance) = self.0.upgrade() {
            if let Ok(mut instance) = instance.try_borrow_mut() {
                instance.mount_status = MountStatus::Unmounted;
            }
        }
    }

    pub fn update<Cl>(&self, fn_update: &impl Fn(&mut C) -> Cl)
    where
        Cl: Into<Checklist<C>>,
    {
        let related_comp_updates = {
            let this = self
                .0
                .upgrade()
                .expect_throw("Expect the component instance alive when updating - update()");
            let mut this = this
                .try_borrow_mut()
                .expect_throw("Multiple borrowing occurred on the component instance - update()");

            // Call `fn_update` here to reduce monomorphization on `CompInstance::extra_update()`
            // Otherwise, `extra_update` need another type parameter `fn_update: &impl Fn(&mut C) -> Cl`.
            let (skip_fn_render, commands, related_comp_updates) =
                fn_update(&mut this.state).into().into_parts();
            this.extra_update(skip_fn_render, commands, &self);
            related_comp_updates
        };
        related_comp_updates.execute();
    }

    pub fn update_arg<T, Cl>(&self, arg: T, fn_update: &impl Fn(&mut C, T) -> Cl)
    where
        Cl: Into<Checklist<C>>,
    {
        let related_comp_updates = {
            let this = self
                .0
                .upgrade()
                .expect_throw("Expect the component instance alive when updating - update_arg()");
            let mut this = this.try_borrow_mut().expect_throw(
                "Multiple borrowing occurred on the component instance - update_arg()",
            );

            // Call `fn_update` here to reduce monomorphization on `CompInstance::extra_update()`
            // Otherwise, `extra_update` need another type parameter `fn_update: &impl Fn(&mut C) -> Cl`.
            let (skip_fn_render, commands, related_comp_updates) =
                fn_update(&mut this.state, arg).into().into_parts();
            this.extra_update(skip_fn_render, commands, &self);
            related_comp_updates
        };
        related_comp_updates.execute();
    }

    pub fn update_child_comps<Cl>(&self, fn_update: &impl Fn(&mut C, &C::Components) -> Cl)
    where
        Cl: Into<Checklist<C>>,
    {
        let related_comp_updates = {
            let this = self.0.upgrade().expect_throw(
                "Expect the component instance alive when updating - update_child_comps()",
            );
            let mut this = this.try_borrow_mut().expect_throw(
                "Multiple borrowing occurred on the component instance - update_child_comps()",
            );

            let (state, child_components) = this.state_and_child_components();

            // Call `fn_update` here to reduce monomorphization on `CompInstance::extra_update()`
            // Otherwise, `extra_update` need another type parameter `fn_update: &impl Fn(&mut C) -> Cl`.
            let (skip_fn_render, commands, related_comp_updates) =
                fn_update(state, child_components).into().into_parts();
            this.extra_update(skip_fn_render, commands, &self);
            related_comp_updates
        };
        related_comp_updates.execute();
    }

    pub fn callback<Cl>(&self, fn_update: impl Fn(&mut C) -> Cl) -> impl Fn()
    where
        Cl: Into<Checklist<C>>,
    {
        let comp = self.clone();
        move || comp.update(&fn_update)
    }

    pub fn callback_arg<T, Cl>(&self, fn_update: impl Fn(&mut C, T) -> Cl) -> impl Fn(T)
    where
        Cl: Into<Checklist<C>>,
    {
        let comp = self.clone();
        move |t: T| comp.update_arg(t, &fn_update)
    }

    pub fn callback_child_comps<Cl>(
        &self,
        fn_update: impl Fn(&mut C, &C::Components) -> Cl,
    ) -> impl Fn()
    where
        Cl: Into<Checklist<C>>,
    {
        let comp = self.clone();
        move || comp.update_child_comps(&fn_update)
    }

    pub fn handler<T, Cl>(&self, fn_update: impl Fn(&mut C) -> Cl) -> impl Fn(T)
    where
        Cl: Into<Checklist<C>>,
    {
        let comp = self.clone();
        move |_: T| comp.update(&fn_update)
    }

    pub fn handler_arg<T, Cl>(&self, fn_update: impl Fn(&mut C, T) -> Cl) -> impl Fn(T)
    where
        Cl: Into<Checklist<C>>,
    {
        self.callback_arg(fn_update)
    }

    pub fn handler_child_comps<T, Cl>(
        &self,
        fn_update: impl Fn(&mut C, &C::Components) -> Cl,
    ) -> impl Fn(T)
    where
        Cl: Into<Checklist<C>>,
    {
        let comp = self.clone();
        move |_: T| comp.update_child_comps(&fn_update)
    }
}

impl<C: Component> CompInstance<C> {
    pub(crate) fn render(&mut self, comp: &Comp<C>) {
        self.state.render(
            self.root_element.create_context(
                comp,
                self.child_components
                    .as_ref()
                    .expect_throw("Why child components None?"),
            ),
        );
    }

    fn extra_update(&mut self, skip_fn_render: bool, mut commands: Commands<C>, comp: &Comp<C>) {
        if !skip_fn_render {
            self.render(comp);
        }
        commands.execute(comp, &mut self.state);
    }

    fn state_and_child_components(&mut self) -> (&mut C, &C::Components) {
        let state = &mut self.state;
        let child_components = self
            .child_components
            .as_ref()
            .expect_throw("Why child_components None?");
        (state, child_components)
    }

    pub fn state(&self) -> &C {
        &self.state
    }

    pub(crate) fn is_mounted(&self) -> bool {
        match self.mount_status {
            MountStatus::Mounted => true,
            _ => false,
        }
    }
}

pub trait Components<P: Component> {
    fn new(parent_state: &P, parent_comp: Comp<P>) -> Self;
}

impl<P: Component> Components<P> for () {
    fn new(_: &P, _: Comp<P>) -> Self {}
}

pub type ChildComp<C> = RcComp<C>;

impl<C: Component> ChildComp<C> {
    // Attach the component to the given ws_element, and run the render
    pub(crate) fn mount_to(&self, ws_element: &web_sys::Element) {
        let comp = self.comp();

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

    pub fn comp_instance(&self) -> std::cell::Ref<CompInstance<C>> {
        self.0.borrow()
    }
}

impl<C: Component> From<C> for ChildComp<C> {
    fn from(state: C) -> Self {
        RcComp::with_state_and_element(state, None)
    }
}

// A new struct instead of impl Drop on Comp because we only want
// to set status to unmounted when removing it from its parent.
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

impl<C: Component> Drop for ChildComp<C> {
    fn drop(&mut self) {
        self.0
            .try_borrow_mut()
            .expect_throw("Why unable to borrow a child component in dropping?")
            .root_element
            .ws_element()
            .set_text_content(None);
    }
}
