use wasm_bindgen::UnwrapThrowExt;

use crate::queue_render::{base::QrListRender, vec::QrVec};

use crate::{
    component::Component,
    dom::ELementTag,
    render::{
        base::{ElementRender, NodesRenderMut},
        html::{
            AttributesOnly, HtmlElementRender, NodesOwned, StaticAttributes, StaticAttributesOnly,
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
            ELementTag::Html(tag),
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
        list.queue_me();
    }
    /*
        fn lwr_clone<I, II, R>(self, items: II, tag: &'a str, render: R) -> NodesExtensions<'a>
        where
            I: Copy,
            II: IntoIterator<Item = I>,
            for<'r> R: Fn(I, crate::Element<'r, C>),
        {
            self.list_with_render(items, ListElementCreation::Clone, tag, render)
        }

        fn list<I, II>(self, items: II, mode: ListElementCreation) -> NodesExtensions<'a>
        where
            I: Copy + ListItemRender<C>,
            II: IntoIterator<Item = I>,
        {
            self.list_with_render(items, mode, I::ROOT_ELEMENT_TAG, I::render)
        }

        fn list_clone<I, II>(self, items: II) -> NodesExtensions<'a>
        where
            I: Copy + ListItemRender<C>,
            II: IntoIterator<Item = I>,
        {
            self.list_with_render(
                items,
                ListElementCreation::Clone,
                I::ROOT_ELEMENT_TAG,
                I::render,
            )
        }
    */
}

impl<'a, C: Component> HemsForQrList<'a, C> for HtmlElementRender<'a, C> {}
impl<'a, C: Component> HemsForQrList<'a, C> for AttributesOnly<'a, C> {}
impl<'a, C: Component> HemsForQrList<'a, C> for StaticAttributes<'a, C> {}
impl<'a, C: Component> HemsForQrList<'a, C> for StaticAttributesOnly<'a, C> {}
