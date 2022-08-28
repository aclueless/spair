use wasm_bindgen::UnwrapThrowExt;

use crate::{queue_render::vec::QrVec, render::html::ListItemRenderRef};

use crate::{
    component::Component,
    render::{
        base::{ElementRender, NodesRenderMut},
        html::{
            AttributesOnly, HtmlElementRender, HtmlTag, NodesOwned, StaticAttributes,
            StaticAttributesOnly,
        },
        ListElementCreation,
    },
};

pub trait HemsForQrList<'a, C: Component>: Sized + Into<NodesOwned<'a, C>> {
    fn qr_list_with_render<I, R>(
        self,
        list: &QrVec<I>,
        mode: ListElementCreation,
        tag: &'static str,
        render: R,
    ) where
        I: 'static,
        for<'i, 'r> R: 'static + Fn(&'i I, crate::Element<'r, C>),
    {
        let mut nodes_render: NodesOwned<C> = self.into();
        let qr_list_render = match nodes_render.nodes_render_mut().create_qr_list_render(
            true,
            mode,
            HtmlTag(tag),
            move |item: &I, er: ElementRender<C>| render(item, er.into()),
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

    fn qr_lwr_clone<I, R>(self,
        list: &QrVec<I>,
        tag: &'static str,
        render: R,
    ) where
        I: 'static,
        for<'i, 'r> R: 'static + Fn(&'i I, crate::Element<'r, C>),
    {
        self.qr_list_with_render(list, ListElementCreation::Clone, tag, render)
    }

    fn qr_list<I: 'static>(self, list: &QrVec<I>, mode: ListElementCreation)
    where
        I: ListItemRenderRef<C>,
    {
        self.qr_list_with_render(list, mode, I::ROOT_ELEMENT_TAG, I::render)
    }

    fn qr_list_clone<I: 'static>(self, list: &QrVec<I>)
    where
        I: ListItemRenderRef<C>,
    {
        self.qr_list_with_render(
            list,
            ListElementCreation::Clone,
            I::ROOT_ELEMENT_TAG,
            I::render,
        )
    }
}

impl<'a, C: Component> HemsForQrList<'a, C> for HtmlElementRender<'a, C> {}
impl<'a, C: Component> HemsForQrList<'a, C> for AttributesOnly<'a, C> {}
impl<'a, C: Component> HemsForQrList<'a, C> for StaticAttributes<'a, C> {}
impl<'a, C: Component> HemsForQrList<'a, C> for StaticAttributesOnly<'a, C> {}
