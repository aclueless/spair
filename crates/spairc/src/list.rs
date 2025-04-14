use crate::{Component, Context, TemplateElement, WsElement};

pub trait ListItemView<C: Component> {
    type ViewState;
    fn template_string() -> &'static str;
    fn create_view(
        template: &TemplateElement,
        cdata: &Self,
        ccontext: &Context<C>,
    ) -> Self::ViewState;
    fn update_view(view_state: &mut Self::ViewState, udata: &Self, ucontext: &Context<C>);
    fn root_element(view_state: &Self::ViewState) -> &WsElement;
}

pub struct List<C, I>
where
    I: ListItemView<C>,
    C: Component,
{
    parent_element: WsElement,
    template: TemplateElement,
    end_node_marker_for_partial_list: Option<web_sys::Node>,
    items: Vec<I::ViewState>,
}

impl<C, I> List<C, I>
where
    I: ListItemView<C> + 'static,
    C: Component + 'static,
{
    pub fn new(
        parent_element: WsElement,
        end_node_marker_for_partial_list: Option<web_sys::Node>,
    ) -> Self {
        Self {
            parent_element,
            template: TemplateElement::new(I::template_string()),
            end_node_marker_for_partial_list,
            items: Vec::new(),
        }
    }

    pub fn update<'a>(&mut self, item_data: impl Iterator<Item = &'a I>, context: &Context<C>) {
        let mut index = 0;
        for item_data in item_data {
            if index >= self.items.len() {
                let mut new_item = I::create_view(&self.template, item_data, context);
                I::update_view(&mut new_item, item_data, context);
                self.parent_element.insert_new_node_before_a_node(
                    I::root_element(&new_item),
                    self.end_node_marker_for_partial_list.as_ref(),
                );
                self.items.push(new_item);
            } else {
                let old_item = unsafe { self.items.get_unchecked_mut(index) };
                I::update_view(old_item, item_data, context);
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
                self.parent_element.remove_child(I::root_element(&item));
            }
        }
    }
}
