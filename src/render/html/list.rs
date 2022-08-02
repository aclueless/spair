use super::ListItemRender;
use crate::component::Component;
use crate::render::base::{
    ElementRender, ElementRenderMut, ListRender, MakeNodesExtensions, NodesExtensions,
    RememberSettingSelectedOption,
};
use crate::render::html::{
    AttributesOnly, HtmlElementRender, HtmlNameSpace, StaticAttributes, StaticAttributesOnly,
};
use crate::render::ListElementCreation;

pub struct HtmlListRender<'a, C: Component>(ListRender<'a, C>);

// TODO: Is it possible to merge this and SvgListRender into ListRender using generic

impl<'a, C: Component> HtmlListRender<'a, C> {
    pub fn render_list<I, R>(
        &mut self,
        items: impl IntoIterator<Item = I>,
        render: R,
    ) -> RememberSettingSelectedOption
    where
        I: Copy,
        for<'u> R: Fn(I, HtmlElementRender<'u, C>),
    {
        let mut index = 0;
        for item in items {
            let status = self
                .0
                .list
                .check_or_create_element_for_list::<HtmlNameSpace>(
                    self.0.tag,
                    index,
                    self.0.parent,
                    self.0.next_sibling,
                    self.0.use_template,
                );
            let element = self.0.list.get_element_mut(index);
            let u = ElementRender::new(self.0.comp, self.0.state, element, status);
            render(item, u.into());
            index += 1;
        }
        self.0.clear_after(index);
        RememberSettingSelectedOption
    }
}

pub trait HtmlListMethods<'a, C: Component>:
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
        let r = self.element_render_mut().non_keyed_list_updater(mode, tag);
        let mut r = HtmlListRender(r);
        let _do_we_have_to_care_about_this_returned_value_ = r.render_list(items, render);
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

    fn lwr_new<I, R>(
        self,
        items: impl IntoIterator<Item = I>,
        tag: &'a str,
        render: R,
    ) -> NodesExtensions<'a>
    where
        I: Copy,
        for<'u> R: Fn(I, crate::Element<'u, C>),
    {
        self.list_with_render(items, ListElementCreation::New, tag, render)
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

impl<'a, C: Component> HtmlListMethods<'a, C> for HtmlElementRender<'a, C> {}
impl<'a, C: Component> HtmlListMethods<'a, C> for AttributesOnly<'a, C> {}
impl<'a, C: Component> HtmlListMethods<'a, C> for StaticAttributes<'a, C> {}
impl<'a, C: Component> HtmlListMethods<'a, C> for StaticAttributesOnly<'a, C> {}
