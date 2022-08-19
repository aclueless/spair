use crate::dom::Nodes;

pub trait MakeNodesExtensions<'a> {
    fn make_nodes_extensions(self) -> NodesExtensions<'a>;
}

pub struct NodesExtensions<'a>(&'a Nodes);

impl<'a> NodesExtensions<'a> {
    pub(crate) fn new(nodes: &'a Nodes) -> Self {
        Self(nodes)
    }

    pub fn done(self) {}

    pub fn scroll_to_last_element_if(
        self,
        need_to_scroll: bool,
        options: &web_sys::ScrollIntoViewOptions,
    ) -> Self {
        if need_to_scroll {
            if let Some(e) = self.0.get_last_element() {
                e.ws_element().scroll_to_view_with_options(options);
            }
        }
        self
    }

    pub fn scroll_to_top_of_last_element_if(self, need_to_scroll: bool) -> Self {
        if need_to_scroll {
            if let Some(e) = self.0.get_last_element() {
                e.ws_element().scroll_to_view_with_bool(true);
            }
        }
        self
    }

    pub fn scroll_to_bottom_of_last_element_if(self, need_to_scroll: bool) -> Self {
        if need_to_scroll {
            if let Some(e) = self.0.get_last_element() {
                e.ws_element().scroll_to_view_with_bool(false);
            }
        }
        self
    }

    pub fn scroll_to_first_element_if(
        self,
        need_to_scroll: bool,
        options: &web_sys::ScrollIntoViewOptions,
    ) -> Self {
        if need_to_scroll {
            if let Some(e) = self.0.get_first_element() {
                e.ws_element().scroll_to_view_with_options(options);
            }
        }
        self
    }

    pub fn scroll_to_top_of_first_element_if(self, need_to_scroll: bool) -> Self {
        if need_to_scroll {
            if let Some(e) = self.0.get_first_element() {
                e.ws_element().scroll_to_view_with_bool(true);
            }
        }
        self
    }

    pub fn scroll_to_bottom_of_first_element_if(self, need_to_scroll: bool) -> Self {
        if need_to_scroll {
            if let Some(e) = self.0.get_first_element() {
                e.ws_element().scroll_to_view_with_bool(false);
            }
        }
        self
    }
}
