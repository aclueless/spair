use crate::queue_render::QrVec;

pub trait HemsForQrList<'a, C: Component>:
    Sized + ElementRenderMut<C> + MakeNodesExtensions<'a>
{
    fn qr_list_with_render<I>(
        mut self,
        items: &QrVec<I>,
        mode: ListElementCreation,
        tag: &'static str,
        render: R,
    )
    where
        for<'r> R: Fn(&I, crate::Element<'r, C>),
    {
        let mut r = self.element_render_mut().qr_list(mode, tag, render);
        let _do_we_have_to_care_about_this_returned_value_ = r
            .render::<HtmlNameSpace, _, _, _>(items, |item: I, er: ElementRender<C>| {
                render(item, er.into())
            });

        self.make_nodes_extensions()
    }
}
