use super::ListItemRender;
use crate::component::Component;
use crate::render::base::{ElementRender, ElementRenderMut, MakeNodesExtensions, NodesExtensions};
use crate::render::html::{
    AttributesOnly, HtmlElementRender, HtmlNameSpace, StaticAttributes, StaticAttributesOnly,
};
use crate::render::ListElementCreation;

pub trait HemsForList<'a, C: Component>:
    Sized + ElementRenderMut<C> + MakeNodesExtensions<'a>
{
    fn list_with_render<I, II, R>(
        mut self,
        items: II,
        mode: ListElementCreation,
        tag: &'a str,
        render: R,
    ) -> NodesExtensions<'a>
    where
        I: Copy,
        II: IntoIterator<Item = I>,
        for<'u> R: Fn(I, crate::Element<'u, C>),
    {
        let mut r = self.element_render_mut().list_render(mode, tag);
        let _do_we_have_to_care_about_this_returned_value_ = r
            .render::<HtmlNameSpace, _, _, _>(items, |item: I, er: ElementRender<C>| {
                render(item, er.into())
            });

        self.make_nodes_extensions()
    }

    fn lwr_clone<I, II, R>(self, items: II, tag: &'a str, render: R) -> NodesExtensions<'a>
    where
        I: Copy,
        II: IntoIterator<Item = I>,
        for<'u> R: Fn(I, crate::Element<'u, C>),
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
