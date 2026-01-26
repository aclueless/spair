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
        // get_view_state_root_element_fn: fn(&VS) -> &WsElement,
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
                self.parent_element.remove_child(
                    item.root_element(), // (get_view_state_root_element_fn)(&item)
                );
            }
        }
    }
}
