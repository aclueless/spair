use web_sys::DocumentFragment;

use crate::{TemplateElement, WsElement};

use crate::keyed_list::ItemViewState;

pub struct List<VS> {
    parent_element: WsElement,
    template: TemplateElement,
    end_node_marker_for_partial_list: Option<web_sys::Node>,

    items: Vec<VS>,
}

impl<VS> List<VS>
where
    VS: ItemViewState,
{
    pub fn new(
        parent_element: &WsElement,
        end_node_marker_for_partial_list: Option<web_sys::Node>,
        template_string: &str,
    ) -> Self {
        Self {
            parent_element: parent_element.clone(),
            template: TemplateElement::new(template_string),
            end_node_marker_for_partial_list,

            items: Vec::new(),
        }
    }

    pub fn end_node(&self) -> Option<&web_sys::Node> {
        self.end_node_marker_for_partial_list.as_ref()
    }

    pub fn update<'a, I>(
        &mut self,
        item_data: impl Iterator<Item = &'a I>,
        create_view_fn: impl Fn(DocumentFragment, &'a I) -> VS,
        update_view_fn: impl Fn(&mut VS, &'a I),
    ) where
        I: 'a,
    {
        let mut index = 0;
        for item_data in item_data {
            if index >= self.items.len() {
                let mut new_item = (create_view_fn)(self.template.fragment_clone(), item_data);
                (update_view_fn)(&mut new_item, item_data);
                self.parent_element.insert_new_node_before_a_node(
                    new_item.root_element(),
                    // (get_view_state_root_element_fn)(&new_item),
                    self.end_node_marker_for_partial_list.as_ref(),
                );
                self.items.push(new_item);
            } else {
                let old_item = unsafe { self.items.get_unchecked_mut(index) };
                (update_view_fn)(old_item, item_data);
            }
            index += 1;
        }

        if index >= self.items.len() {
            return;
        }
        if index == 0 && self.end_node_marker_for_partial_list.is_none() {
            self.parent_element.clear_text_content();
            self.items.clear();
        } else {
            for item in self.items.drain(index..) {
                self.parent_element.remove_child(item.root_element());
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[cfg(test)]
mod tests {
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::{Text, WsElement, WsNodeFns};
    #[wasm_bindgen_test]
    fn test_list() {
        struct TestElement(WsElement);
        impl TestElement {
            fn texts(&self) -> Vec<String> {
                let mut texts = Vec::new();
                let mut node = match self.0.get_ws_node_ref().first_child() {
                    Some(node) => node,
                    None => return Vec::new(),
                };
                texts.push(node.get_ws_node_ref().text_content().unwrap());
                while let Some(next) = node.next_sibling() {
                    texts.push(next.text_content().unwrap());
                    node = next;
                }
                texts
            }
        }
        trait RefStr {
            fn to_ref(&self) -> Vec<&str>;
        }
        impl RefStr for Vec<String> {
            fn to_ref(&self) -> Vec<&str> {
                self.iter().map(|v| v.as_str()).collect()
            }
        }
        struct ViewState {
            element: WsElement,
            text: Text,
        }
        impl super::ItemViewState for ViewState {
            fn root_element(&self) -> &WsElement {
                &self.element
            }
        }
        let element = TestElement(WsElement::create_element("div"));
        let mut list = super::List::new(&element.0, None, "<span>?</span>");

        let empty: [&str; 0] = [];
        assert_eq!(&empty[..], &element.texts().to_ref());

        let create_view = |df: web_sys::DocumentFragment, _cd| {
            let element = df.first_ws_element();
            let text = element.first_text();
            ViewState { element, text }
        };
        let update_view = |vs: &mut ViewState, ud: &&str| {
            vs.text.update(*ud);
        };

        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"];
        list.update(data.iter(), create_view, update_view);
        assert_eq!(&data, &element.texts().to_ref());

        // Random shuffle + addition
        let data = vec!["f", "b", "d", "l", "g", "i", "m", "j", "a", "h", "k"];
        list.update(data.iter(), create_view, update_view);
        assert_eq!(&data, &element.texts().to_ref());

        // Empty the list
        list.update(empty.iter(), create_view, update_view);
        assert_eq!(&empty[..], &element.texts().to_ref());

        // Add back
        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"];
        list.update(data.iter(), create_view, update_view);
        assert_eq!(&data, &element.texts().to_ref());

        // Forward
        let data = vec!["a", "i", "b", "c", "d", "e", "f", "g", "h", "j", "k"];
        list.update(data.iter(), create_view, update_view);
        assert_eq!(&data, &element.texts().to_ref());

        // Backward
        let data = vec!["a", "i", "c", "d", "e", "f", "g", "h", "b", "j", "k"];
        list.update(data.iter(), create_view, update_view);
        assert_eq!(&data, &element.texts().to_ref());

        // Swap
        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"];
        list.update(data.iter(), create_view, update_view);
        assert_eq!(&data, &element.texts().to_ref());

        // Remove middle
        let data = vec!["a", "b", "c", "d", "i", "j", "k"];
        list.update(data.iter(), create_view, update_view);
        assert_eq!(&data, &element.texts().to_ref());

        // Insert middle
        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"];
        list.update(data.iter(), create_view, update_view);
        assert_eq!(&data, &element.texts().to_ref());

        // Remove start
        let data = vec!["d", "e", "f", "g", "h", "i", "j", "k"];
        list.update(data.iter(), create_view, update_view);
        assert_eq!(&data, &element.texts().to_ref());

        // Insert start
        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"];
        list.update(data.iter(), create_view, update_view);
        assert_eq!(&data, &element.texts().to_ref());

        // Remove end
        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h"];
        list.update(data.iter(), create_view, update_view);
        assert_eq!(&data, &element.texts().to_ref());

        // Append end
        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"];
        list.update(data.iter(), create_view, update_view);
        assert_eq!(&data, &element.texts().to_ref());
    }
}
