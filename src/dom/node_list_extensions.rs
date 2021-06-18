pub struct NodeListExtensions<'a>(pub(super) &'a crate::dom::nodes::NodeList);

impl<'a> NodeListExtensions<'a> {
    pub fn scroll_to_last_item_with_if(
        self,
        need_to_scroll: bool,
        options: &web_sys::ScrollIntoViewOptions,
    ) -> Self {
        if need_to_scroll {
            if let Some(e) = self.0.get_last_element() {
                e.scroll_to_view_with_options(options);
            }
        }
        self
    }

    pub fn scroll_to_top_of_last_element_if(self, need_to_scroll: bool) -> Self {
        if need_to_scroll {
            if let Some(e) = self.0.get_last_element() {
                e.scroll_to_view_with_bool(true);
            }
        }
        self
    }

    pub fn scroll_to_bottom_of_last_element_if(self, need_to_scroll: bool) -> Self {
        if need_to_scroll {
            if let Some(e) = self.0.get_last_element() {
                e.scroll_to_view_with_bool(false);
            }
        }
        self
    }
}
