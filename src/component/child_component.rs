use wasm_bindgen::UnwrapThrowExt;

use super::{Checklist, Comp, CompInstance, Component, RcComp};
use crate::dom::{Element, NameSpace};

pub type ChildComp<C> = RcComp<C>;

impl<C: Component> ChildComp<C> {
    // Attach the component to the given ws_element, and run the render
    /*
    pub(crate) fn mount_to(&self, ws_element: &web_sys::Element) {
        let comp = self.comp();

        C::init(&comp);

        let promise = super::i_have_to_execute_update_queue();

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
        super::execute_update_queue(promise);
    }
    */

    pub fn comp_instance(&self) -> std::cell::Ref<CompInstance<C>> {
        self.0.borrow()
    }
}

// impl<C: Component> From<C> for ChildComp<C> {
//     fn from(state: C) -> Self {
//         let rc_comp = ChildComp::with_ws_root(None);
//         rc_comp.set_state(state);
//         rc_comp
//     }
// }

pub enum ELementTag {
    Html(&'static str),
    #[cfg(feature = "svg")]
    Svg(&'static str),
}

pub trait AsChildComp: Sized + Component {
    const ROOT_ELEMENT_TAG: ELementTag;
    type Properties;
    fn init(comp: &Comp<Self>, props: Self::Properties) -> Self;
}

impl<C: AsChildComp + Component> ChildComp<C> {
    pub fn with_props(props: C::Properties) -> Self {
        let root_element = match C::ROOT_ELEMENT_TAG {
            ELementTag::Html(tag) => {
                Element::new_ns(crate::render::html::HtmlNameSpace::NAMESPACE, tag)
            }
            #[cfg(feature = "svg")]
            ELementTag::Svg(tag) => {
                Element::new_ns(crate::render::svg::SvgNameSpace::NAMESPACE, tag)
            }
        };
        let rc_comp = ChildComp::with_root(root_element);
        let comp = rc_comp.comp();
        let state = AsChildComp::init(&comp, props);
        rc_comp.set_state(state);
        crate::routing::register_routing_callback(&comp);
        rc_comp
    }

    pub fn with_updater<P, T, G, U, Cl>(self, fn_get_value: G, cb: U) -> Child<P, C, T>
    where
        P: Component,
        G: 'static + Fn(&P) -> &T,
        U: 'static + Fn(&mut C, T) -> Cl,
        Cl: 'static + Into<Checklist<C>>,
        T: 'static + Clone + PartialEq,
    {
        let child_callback = self.comp().callback_arg_mut(cb);
        Child {
            child: self,
            last_value: None,
            fn_get_value: Some(Box::new(fn_get_value)),
            child_callback: Some(child_callback),
        }
    }

    pub fn no_updater<P, G, U, Cl>(self) -> Child<P, C, ()>
    where
        P: Component,
    {
        Child {
            child: self,
            last_value: None,
            fn_get_value: None,
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
        self.0
            .try_borrow_mut()
            .expect_throw("Why unable to borrow a child component in dropping?")
            .root_element
            .ws_element()
            .set_text_content(None);
    }
}

// pub trait ChildOf<P: Component> {
//     fn update(&self, parent_state: &P);
// }

pub struct Child<P, C, T>
where
    P: Component,
    C: Component,
    T: Clone + PartialEq,
{
    child: ChildComp<C>,
    last_value: Option<T>,
    fn_get_value: Option<Box<dyn Fn(&P) -> &T>>,
    child_callback: Option<crate::CallbackArg<T>>,
}

impl<P, C, T> Child<P, C, T>
where
    P: Component,
    C: Component,
    T: Clone + PartialEq,
{
    // This return `true` if it queue an update
    pub fn update(&mut self, parent_state: &P) -> bool {
        let getter = match self.fn_get_value.as_ref() {
            Some(g) => g,
            None => return false,
        };

        let new_value = (getter)(parent_state);
        if let Some(old_value) = self.last_value.as_ref() {
            if new_value == old_value {
                return false;
            }
        }
        self.last_value = Some(new_value.clone());

        if let Some(cb) = self.child_callback.as_ref() {
            cb.queue(new_value.clone());
            return true;
        }
        false
    }

    pub fn get_root_node(&self) -> web_sys::Node {
        let v =
            self.child.0.try_borrow().expect_throw(
                "component::child_component::Child::get_root_node borrow CompInstance",
            );
        let n: &web_sys::Node = v.root_element.ws_element().as_ref();
        n.clone()
    }

    pub fn first_render(&self) {
        self.child.first_render();
    }
}

// impl<P: Component, C: Component, T> ChildOf<P> for Child<P, C, T>
// {
//     fn update(&mut self, parent_state: &P) {
//
//     }
// }
/*
pub struct ChildComponent<P, C: AsChildComp + Component>{
    child: ChildComp<C>,
    updaters: Vec<Box<dyn UpdateChildComponent<P, C>>>,
}

trait UpdateChildComponent<P, C> {
    fn update(&self, p: &P, c: &mut C);
}

struct Updater<G, S> {
    getter: G,
    setter: S,
}

impl<C: AsChildComp + Component> NewChildComp<C> {
    fn add_updater(mut self) -> Self {
        self.updaters.push(value);
        self
    }
}
*/
/*
            .div(|d| {
                d.component(
                    spair::ChildComp::init::<Self>()
                        .set_props(|state, comp| {})
                        .set_updaters(|updaters| {
                            updaters
                                .add(State::get_value, ChildState::set_value)
                                .add(|_| (), ChildState.tick);
                        }),
                )
            })
*/
