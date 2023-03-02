use wasm_bindgen::UnwrapThrowExt;

use crate::queue_render::vec::QrVec;

use crate::{
    component::Component,
    render::{
        base::{ElementUpdater, NodesUpdaterMut},
        svg::{
            SvgAttributesOnly, SvgElementUpdater, SvgNodesOwned, SvgStaticAttributes,
            SvgStaticAttributesOnly, SvgTag,
        },
        ListElementCreation,
    },
};

pub trait SemsForQrList<'a, C: Component>: Sized + Into<SvgNodesOwned<'a, C>> {
    fn qr_list<I, R>(self, list: &QrVec<I>, mode: ListElementCreation, tag: &'static str, render: R)
    where
        I: 'static + Clone,
        R: 'static + Fn(I, crate::SvgElement<C>),
    {
        let mut nodes_updater: SvgNodesOwned<C> = self.into();
        let qr_list_render = match nodes_updater.nodes_updater_mut().create_qr_list_render(
            true,
            mode,
            SvgTag(tag),
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
        R: 'static + Fn(I, crate::SvgElement<C>),
    {
        self.qr_list(list, ListElementCreation::Clone, tag, render)
    }
}

impl<'a, C: Component> SemsForQrList<'a, C> for SvgElementUpdater<'a, C> {}
impl<'a, C: Component> SemsForQrList<'a, C> for SvgAttributesOnly<'a, C> {}
impl<'a, C: Component> SemsForQrList<'a, C> for SvgStaticAttributes<'a, C> {}
impl<'a, C: Component> SemsForQrList<'a, C> for SvgStaticAttributesOnly<'a, C> {}
