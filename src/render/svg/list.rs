use super::SvgListItemRender;
use crate::component::Component;
use crate::render::base::{
    ElementRender, ElementRenderMut, ListRender, MakeNodesExtensions, NodesExtensions,
    RememberSettingSelectedOption,
};
use crate::render::svg::{
    SvgAttributesOnly, SvgElementRender, SvgNameSpace, SvgStaticAttributes, SvgStaticAttributesOnly,
};
use crate::render::ListElementCreation;

pub struct SvgListRender<'a, C: Component>(ListRender<'a, C>);

// TODO: Is it possible to merge this and HtmlListRender into ListRender using generic?

impl<'a, C: Component> SvgListRender<'a, C> {
    pub fn new(lr: ListRender<'a, C>) -> Self {
        Self(lr)
    }

    pub fn render_list<I, R>(
        &mut self,
        items: impl IntoIterator<Item = I>,
        render: R,
    ) -> RememberSettingSelectedOption
    where
        I: Copy,
        for<'u> R: Fn(I, SvgElementRender<'u, C>),
    {
        let mut index = 0;
        for item in items {
            let status = self
                .0
                .list
                .check_or_create_element_for_list::<SvgNameSpace>(
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

pub trait SemsForList<'a, C: Component>:
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
        for<'u> R: Fn(I, crate::Svg<'u, C>),
    {
        let r = self.element_render_mut().list_render(mode, tag);
        let mut r = SvgListRender(r);
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
        for<'u> R: Fn(I, crate::Svg<'u, C>),
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
        for<'u> R: Fn(I, crate::Svg<'u, C>),
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
        I: SvgListItemRender<C>,
    {
        self.list_with_render(items, mode, I::ROOT_ELEMENT_TAG, I::render)
    }

    fn list_clone<I>(self, items: impl IntoIterator<Item = I>) -> NodesExtensions<'a>
    where
        I: Copy,
        I: SvgListItemRender<C>,
    {
        self.list_with_render(
            items,
            ListElementCreation::Clone,
            I::ROOT_ELEMENT_TAG,
            I::render,
        )
    }
}

impl<'a, C: Component> MakeNodesExtensions<'a> for SvgElementRender<'a, C> {
    fn make_nodes_extensions(self) -> NodesExtensions<'a> {
        let e = self.into_inner().into_parts().3;
        NodesExtensions::new(e.nodes_mut())
    }
}

impl<'a, C: Component> MakeNodesExtensions<'a> for SvgAttributesOnly<'a, C> {
    fn make_nodes_extensions(self) -> NodesExtensions<'a> {
        let e = self.into_inner().into_parts().3;
        NodesExtensions::new(e.nodes_mut())
    }
}

impl<'a, C: Component> MakeNodesExtensions<'a> for SvgStaticAttributes<'a, C> {
    fn make_nodes_extensions(self) -> NodesExtensions<'a> {
        let e = self.into_inner().into_parts().3;
        NodesExtensions::new(e.nodes_mut())
    }
}

impl<'a, C: Component> MakeNodesExtensions<'a> for SvgStaticAttributesOnly<'a, C> {
    fn make_nodes_extensions(self) -> NodesExtensions<'a> {
        let e = self.into_inner().into_parts().3;
        NodesExtensions::new(e.nodes_mut())
    }
}

impl<'a, C: Component> SemsForList<'a, C> for SvgElementRender<'a, C> {}
impl<'a, C: Component> SemsForList<'a, C> for SvgAttributesOnly<'a, C> {}
impl<'a, C: Component> SemsForList<'a, C> for SvgStaticAttributes<'a, C> {}
impl<'a, C: Component> SemsForList<'a, C> for SvgStaticAttributesOnly<'a, C> {}
