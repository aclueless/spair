use std::cell::RefCell;
use std::rc::{Rc, Weak};
use wasm_bindgen::UnwrapThrowExt;

pub trait Component: 'static + Sized {
    type Routes: crate::routing::Routes<Self>;
    // fn init() -> Self;
    fn render<'a>(&self, context: Context<'a, Self>);
}

pub struct Context<'a, C> {
    pub element: crate::dom::ElementHandle<'a, C>,
    pub comp: &'a Comp<C>,
}

impl<'a, C> Context<'a, C> {
    pub fn new(comp: &'a Comp<C>, element: crate::dom::ElementHandle<'a, C>) -> Self {
        Self { comp, element }
    }
    pub fn into_parts(self) -> (&'a Comp<C>, crate::dom::ElementHandle<'a, C>) {
        (self.comp, self.element)
    }
}

pub struct RcComp<C>(Rc<RefCell<CompInstance<C>>>);
pub struct Comp<C>(Weak<RefCell<CompInstance<C>>>);

struct CompInstance<C> {
    state: C,
    root_element: crate::dom::Element,
    router: Option<crate::routing::Router>,
}

pub struct Checklist<C: Component> {
    skip_fn_render: bool,
    commands: Vec<Box<dyn Command<C>>>,
}

pub trait Command<C: Component> {
    fn execute(&mut self, comp: &Comp<C>, state: &mut C);
}

impl<C: Component> Default for Checklist<C> {
    fn default() -> Self {
        Self {
            skip_fn_render: false,
            commands: Vec::new(),
        }
    }
}

impl<C: Component> From<()> for Checklist<C> {
    fn from(_: ()) -> Self {
        Self::default()
    }
}

impl<C: Component> Checklist<C> {
    pub fn skip_fn_render() -> Self {
        let mut s = Self::default();
        s.skip_fn_render = true;
        s
    }

    fn execute_commands(&mut self, comp: &Comp<C>, state: &mut C) {
        self.commands
            .iter_mut()
            .for_each(|c| c.execute(comp, state));
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
        self.commands.push(Box::new(crate::fetch::FetchCommand::new(
            req, options, ok, error,
        )));
    }
}

impl<C: Component> RcComp<C> {
    pub fn new(state: C, root: web_sys::Element) -> Self {
        Self(Rc::new(RefCell::new(CompInstance {
            state,
            root_element: crate::dom::Element::from_ws_element(root),
            router: None,
        })))
    }

    pub fn first_render(&self) {
        use crate::routing::Routes;
        let comp = Comp(Rc::downgrade(&self.0));
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
}

impl<C> Clone for Comp<C> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<C: Component> Comp<C> {
    pub fn update<Cl>(&self, fn_update: &impl Fn(&mut C) -> Cl)
    where
        Cl: Into<Checklist<C>>,
    {
        let this = self
            .0
            .upgrade()
            .expect_throw("Expect the component instance alive when updating");
        let mut this = this
            .try_borrow_mut()
            .expect_throw("Multiple borrowing occurred on the component instance");

        // Call `fn_update` here to reduce monomorphization on `CompInstance::extra_update()`
        // Otherwise, `extra_update` need another type parameter `fn_update: &impl Fn(&mut C) -> Cl`.
        let checklist = fn_update(&mut this.state);
        this.extra_update(checklist.into(), &self);
    }

    pub fn update_arg<T, Cl>(&self, arg: T, fn_update: &impl Fn(&mut C, T) -> Cl)
    where
        Cl: Into<Checklist<C>>,
    {
        let this = self
            .0
            .upgrade()
            .expect_throw("Expect the component instance alive when updating");
        let mut this = this
            .try_borrow_mut()
            .expect_throw("Multiple borrowing occurred on the component instance");

        // Call `fn_update` here to reduce monomorphization on `CompInstance::extra_update()`
        // Otherwise, `extra_update` need another type parameter `fn_update: &impl Fn(&mut C) -> Cl`.
        let checklist = fn_update(&mut this.state, arg);
        this.extra_update(checklist.into(), &self);
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
        let comp = self.clone();
        move |t: T| comp.update_arg(t, &fn_update)
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
        self.handler_arg(fn_update)
    }
}

impl<C: Component> CompInstance<C> {
    pub fn render(&mut self, comp: &Comp<C>) {
        self.state.render(self.root_element.create_context(comp));
    }

    fn extra_update(&mut self, mut checklist: Checklist<C>, comp: &Comp<C>) {
        if !checklist.skip_fn_render {
            self.render(comp);
        }
        checklist.execute_commands(comp, &mut self.state);
    }
}
