use wasm_bindgen::UnwrapThrowExt;

use crate::queue_render::vec::QrVec;

use crate::{
    component::Component,
    render::{
        base::{ElementUpdater, NodesUpdaterMut},
        html::{
            AttributesOnly, HtmlElementUpdater, HtmlTag, NodesOwned, StaticAttributes,
            StaticAttributesOnly,
        },
        ListElementCreation,
    },
};

pub trait HemsForQrList<'a, C: Component>: Sized + Into<NodesOwned<'a, C>> {
    fn qr_list<I, R>(self, list: &QrVec<I>, mode: ListElementCreation, tag: &'static str, render: R)
    where
        I: 'static + Clone,
        R: 'static + Fn(I, crate::Element<C>),
    {
        let mut nodes_updater: NodesOwned<C> = self.into();
        let qr_list_render = match nodes_updater.nodes_updater_mut().create_qr_list_render(
            true,
            mode,
            HtmlTag(tag),
            move |item: I, er: ElementUpdater<C>| render(item, er.into()),
        ) {
            None => return,
            Some(render) => render,
        };
        list.content()
            .try_borrow_mut()
            .expect_throw("queue_render::html::list::HemsForQrList::qr_list content borrow mut")
            .add_render(Box::new(qr_list_render));
        list.check_and_queue_a_render();
    }

    fn qr_list_clone<I, R>(self, list: &QrVec<I>, tag: &'static str, render: R)
    where
        I: 'static + Clone,
        R: 'static + Fn(I, crate::Element<C>),
    {
        self.qr_list(list, ListElementCreation::Clone, tag, render)
    }
}

impl<'a, C: Component> HemsForQrList<'a, C> for HtmlElementUpdater<'a, C> {}
impl<'a, C: Component> HemsForQrList<'a, C> for AttributesOnly<'a, C> {}
impl<'a, C: Component> HemsForQrList<'a, C> for StaticAttributes<'a, C> {}
impl<'a, C: Component> HemsForQrList<'a, C> for StaticAttributesOnly<'a, C> {}
