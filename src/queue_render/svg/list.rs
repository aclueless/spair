use wasm_bindgen::UnwrapThrowExt;

use crate::{queue_render::vec::QrVec, render::svg::SvgElementRender};

use crate::{
    component::Component,
    render::{
        base::{ElementUpdater, NodesRenderMut},
        svg::{
            SvgAttributesOnly, SvgElementUpdater, SvgNodesOwned, SvgStaticAttributes,
            SvgStaticAttributesOnly, SvgTag,
        },
        ListElementCreation,
    },
};

pub trait SemsForQrList<'a, C: Component>: Sized + Into<SvgNodesOwned<'a, C>> {
    fn qr_list_with_render<I: Clone, R>(
        self,
        list: &QrVec<I>,
        mode: ListElementCreation,
        tag: &'static str,
        render: R,
    ) where
        I: 'static,
        //for<'i, 'r> R: 'static + Fn(&'i I, crate::SvgElement<'r, C>),
        R: 'static + Fn(I, crate::SvgElement<C>),
    {
        let mut nodes_render: SvgNodesOwned<C> = self.into();
        let qr_list_render = match nodes_render.nodes_render_mut().create_qr_list_render(
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
            .expect_throw(
                "queue_render::html::list::HemsForQrList::qr_list_with_render content borrow mut",
            )
            .add_render(Box::new(qr_list_render));
        list.check_and_queue_a_render();
    }

    fn qr_lwr_clone<I: Clone, R>(self, list: &QrVec<I>, tag: &'static str, render: R)
    where
        I: 'static,
        //for<'i, 'r> R: 'static + Fn(&'i I, crate::SvgElement<'r, C>),
        R: 'static + Fn(I, crate::SvgElement<C>),
    {
        self.qr_list_with_render(list, ListElementCreation::Clone, tag, render)
    }

    fn qr_list<I: 'static + Clone>(self, list: &QrVec<I>, mode: ListElementCreation)
    where
        I: SvgElementRender<C>,
    {
        self.qr_list_with_render(list, mode, I::ELEMENT_TAG, I::render)
    }

    fn qr_list_clone<I: 'static + Clone>(self, list: &QrVec<I>)
    where
        I: SvgElementRender<C>,
    {
        self.qr_list_with_render(list, ListElementCreation::Clone, I::ELEMENT_TAG, I::render)
    }
}

impl<'a, C: Component> SemsForQrList<'a, C> for SvgElementUpdater<'a, C> {}
impl<'a, C: Component> SemsForQrList<'a, C> for SvgAttributesOnly<'a, C> {}
impl<'a, C: Component> SemsForQrList<'a, C> for SvgStaticAttributes<'a, C> {}
impl<'a, C: Component> SemsForQrList<'a, C> for SvgStaticAttributesOnly<'a, C> {}
