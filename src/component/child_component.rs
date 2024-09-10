use wasm_bindgen::UnwrapThrowExt;

use super::{Checklist, Comp, CompInstance, Component, RcComp};
use crate::dom::{CompRef, ComponentRef, Element, TagName};

pub type ChildComp<C> = RcComp<C>;

impl<C: Component> ChildComp<C> {
    pub fn comp_instance(&self) -> std::cell::Ref<CompInstance<C>> {
        self.0.borrow()
    }

    // Make a ComponentRef and do a first render if it has never rendered before
    pub fn component_ref(&self) -> Option<Box<dyn ComponentRef>> {
        let comp = self.comp();

        // Just render self-component, this component dont have to care about executing
        // the update queue because this method has to be called from a parent component
        // which must take care of the update queue if there is no other component is being
        // updated.

        let mut instance = self
            .0
            .try_borrow_mut()
            .expect_throw("Expect no borrowing at the first render");

        if instance.is_mounted() {
            // nothing to do because the component is already in place.
            return None;
            // otherwise, the component is new, it will be rendered and returned a
            // handle to mount to the DOM
        }

        instance.mount_status = super::ComponentMountStatus::Mounted;
        C::init(&comp);
        if instance.root_element.is_empty() {
            // In cases that the router not cause any render yet, such as Routes = ()
            instance.render(&comp);
        }

        let ws_node = instance.root_element.ws_element().ws_node().clone();
        Some(Box::new(CompRef {
            _comp: comp.into(),
            ws_node,
        }))
    }
}

pub trait AsChildComp: Sized + Component {
    const ROOT_ELEMENT_TAG: TagName;
    type Properties;
    fn init(comp: &Comp<Self>, props: Self::Properties) -> Self;
    fn with_props(props: Self::Properties) -> ChildComp<Self> {
        let root_element = match Self::ROOT_ELEMENT_TAG {
            TagName::Html(tag) => Element::new_ns(tag),
            #[cfg(feature = "svg")]
            TagName::Svg(tag) => Element::new_ns(tag),
        };
        let rc_comp = ChildComp::with_root(root_element);
        let comp = rc_comp.comp();
        let state = AsChildComp::init(&comp, props);
        rc_comp.set_state(state);
        crate::routing::register_routing_callback(&comp);
        rc_comp
    }
}

impl<C: AsChildComp + Component> ChildComp<C> {
    pub fn with_props(props: C::Properties) -> ChildComp<C> {
        C::with_props(props)
    }

    /// Execute update on the child's state and UI with given functions.
    /// The `child_callback` is executed only if the value returned by `value_getter`
    /// changed.
    pub fn with_updater<P, T, G, U, Cl>(self, value_getter: G, child_callback: U) -> Child<P, C, T>
    where
        P: Component,
        G: 'static + Fn(&P) -> T,
        U: 'static + Fn(&mut C, T) -> Cl,
        Cl: 'static + Into<Checklist<C>>,
        T: 'static + Clone + PartialEq,
    {
        let child_callback = self.comp().callback_arg_mut(child_callback);
        Child {
            child: self,
            last_value: None,
            value_getter: Some(Box::new(value_getter)),
            child_callback: Some(child_callback),
        }
    }

    pub fn no_updater<P>(self) -> Child<P, C, ()>
    where
        P: Component,
    {
        Child {
            child: self,
            last_value: None,
            value_getter: None,
            child_callback: None,
        }
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

impl<C: Component> Drop for ChildComp<C> {
    fn drop(&mut self) {
        crate::routing::remove_routing_callback::<C>();
    }
}

type GetValue<P, T> = Box<dyn Fn(&P) -> T>;

pub struct Child<P, C, T>
where
    P: Component,
    C: Component,
    T: Clone + PartialEq,
{
    child: ChildComp<C>,
    last_value: Option<T>,
    value_getter: Option<GetValue<P, T>>,
    child_callback: Option<crate::CallbackArg<T>>,
}

impl<P, C, T> Child<P, C, T>
where
    P: Component,
    C: Component,
    T: Clone + PartialEq,
{
    // This return `true` if it queue an update
    pub(crate) fn update(&mut self, parent_state: &P) -> bool {
        let getter = match self.value_getter.as_ref() {
            Some(g) => g,
            None => return false,
        };

        let new_value = (getter)(parent_state);
        if let Some(old_value) = self.last_value.as_ref() {
            if new_value == *old_value {
                return false;
            }
        }
        self.last_value = Some(new_value.clone());

        if let Some(cb) = self.child_callback.as_ref() {
            cb.queue(new_value);
            return true;
        }
        false
    }

    pub(crate) fn get_root_node(&self) -> web_sys::Node {
        let v =
            self.child.0.try_borrow().expect_throw(
                "component::child_component::Child::get_root_node borrow CompInstance",
            );
        let n: &web_sys::Node = v.root_element.ws_element().ws_node();
        n.clone()
    }

    pub(crate) fn first_render(&self) {
        self.child.first_render();
    }

    pub(crate) fn init(&self) {
        Component::init(&self.child.comp());
    }
}
