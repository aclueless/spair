use super::ListItemRender;
use crate::component::Component;
use crate::dom::NameSpace;
use crate::render::base::{ElementRender, ElementRenderMut, MakeNodesExtensions, NodesExtensions};
use crate::render::html::{
    AttributesOnly, HtmlElementRender, HtmlNameSpace, StaticAttributes, StaticAttributesOnly,
};
use crate::render::ListElementCreation;

pub trait HemsForList<'a, C: Component>:
    Sized + ElementRenderMut<C> + MakeNodesExtensions<'a>
{
    fn list_with_render<I, R>(
        mut self,
        items: impl IntoIterator<Item = I>,
        mode: ListElementCreation,
        tag: &'a str,
        render: R,
    ) -> NodesExtensions<'a>
    where
        I: Copy,
        for<'u> R: Fn(I, crate::Element<'u, C>),
    {
        let mut r = self
            .element_render_mut()
            .list_render(mode, tag, HtmlNameSpace::NAMESPACE);
        let _do_we_have_to_care_about_this_returned_value_ = r
            .render(items, |item: I, er: ElementRender<C>| {
                render(item, er.into())
            });

        self.make_nodes_extensions()
    }

    fn lwr_clone<I, R>(
        self,
        items: impl IntoIterator<Item = I>,
        tag: &'a str,
        render: R,
    ) -> NodesExtensions<'a>
    where
        I: Copy,
        for<'u> R: Fn(I, crate::Element<'u, C>),
    {
        self.list_with_render(items, ListElementCreation::Clone, tag, render)
    }

    fn list<I>(
        self,
        items: impl IntoIterator<Item = I>,
        mode: ListElementCreation,
    ) -> NodesExtensions<'a>
    where
        I: Copy,
        I: ListItemRender<C>,
    {
        self.list_with_render(items, mode, I::ROOT_ELEMENT_TAG, I::render)
    }

    fn list_clone<I>(self, items: impl IntoIterator<Item = I>) -> NodesExtensions<'a>
    where
        I: Copy,
        I: ListItemRender<C>,
    {
        self.list_with_render(
            items,
            ListElementCreation::Clone,
            I::ROOT_ELEMENT_TAG,
            I::render,
        )
    }
}

impl<'a, C: Component> MakeNodesExtensions<'a> for HtmlElementRender<'a, C> {
    fn make_nodes_extensions(self) -> NodesExtensions<'a> {
        let e = self.into_parts().0.into_parts().3;
        NodesExtensions::new(e.nodes_mut())
    }
}

impl<'a, C: Component> MakeNodesExtensions<'a> for AttributesOnly<'a, C> {
    fn make_nodes_extensions(self) -> NodesExtensions<'a> {
        let e = self.into_inner().into_parts().0.into_parts().3;
        NodesExtensions::new(e.nodes_mut())
    }
}

impl<'a, C: Component> MakeNodesExtensions<'a> for StaticAttributes<'a, C> {
    fn make_nodes_extensions(self) -> NodesExtensions<'a> {
        let e = self.into_inner().into_parts().0.into_parts().3;
        NodesExtensions::new(e.nodes_mut())
    }
}

impl<'a, C: Component> MakeNodesExtensions<'a> for StaticAttributesOnly<'a, C> {
    fn make_nodes_extensions(self) -> NodesExtensions<'a> {
        let e = self.into_inner().into_parts().0.into_parts().3;
        NodesExtensions::new(e.nodes_mut())
    }
}

impl<'a, C: Component> HemsForList<'a, C> for HtmlElementRender<'a, C> {}
impl<'a, C: Component> HemsForList<'a, C> for AttributesOnly<'a, C> {}
impl<'a, C: Component> HemsForList<'a, C> for StaticAttributes<'a, C> {}
impl<'a, C: Component> HemsForList<'a, C> for StaticAttributesOnly<'a, C> {}
