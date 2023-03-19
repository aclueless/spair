use wasm_bindgen::UnwrapThrowExt;

use crate::queue_render::vec::QrVec;

use crate::{
    component::Component,
    render::{
        base::{NodesUpdater, NodesUpdaterMut},
        svg::{
            SvgAttributesOnly, SvgElementUpdater, SvgNodesOwned, SvgStaticAttributes,
            SvgStaticAttributesOnly,
        },
        ListElementCreation,
    },
};

pub trait SemsForQrList<'a, C: Component>: Sized + Into<SvgNodesOwned<'a, C>> {
    fn qr_list<I, R>(self, list: &QrVec<I>, mode: ListElementCreation, render: R)
    where
        I: 'static + Clone,
        R: 'static + Fn(I, crate::SvgNodes<C>),
    {
        let mut nodes_updater: SvgNodesOwned<C> = self.into();
        let qr_list_render = match nodes_updater.nodes_updater_mut().create_qr_list_render(
            true,
            mode,
            move |entry: I, mut nodes: NodesUpdater<C>| {
                render(entry, crate::SvgNodes::new(&mut nodes))
            },
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

    fn qr_list_clone<I, R>(self, list: &QrVec<I>, render: R)
    where
        I: 'static + Clone,
        R: 'static + Fn(I, crate::SvgNodes<C>),
    {
        self.qr_list(list, ListElementCreation::Clone, render)
    }
}

impl<'a, C: Component> SemsForQrList<'a, C> for SvgElementUpdater<'a, C> {}
impl<'a, C: Component> SemsForQrList<'a, C> for SvgAttributesOnly<'a, C> {}
impl<'a, C: Component> SemsForQrList<'a, C> for SvgStaticAttributes<'a, C> {}
impl<'a, C: Component> SemsForQrList<'a, C> for SvgStaticAttributesOnly<'a, C> {}
