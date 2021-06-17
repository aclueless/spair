pub struct NodeListExtensions<'a>(pub(super) &'a crate::dom::nodes::NodeList);

impl<'a> NodeListExtensions<'a> {
    fn scroll_to_last_item_if(
        self,
        need_to_scroll: bool,
        options: &web_sys::ScrollIntoViewOptions,
    ) -> Self {
        if need_to_scroll {
            self.0.scroll_to_last_item(options);
        }
        self
    }

    fn scroll_to_top_of_last_element_if(self, need_to_scroll: bool) -> Self {
        if need_to_scroll {
            self.0.scroll_to_last_item_with_bool(true);
        }
        self
    }
}
