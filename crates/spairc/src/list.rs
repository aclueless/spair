use crate::{Component, Context, TemplateElement, WsElement};

pub trait ListItemView<C: Component> {
    type ViewState;
    fn template_string() -> &'static str;
    fn create(template: &TemplateElement, cdata: &Self, ccontext: &Context<C>) -> Self::ViewState;
    fn update(view_state: &mut Self::ViewState, udata: &Self, ucontext: &Context<C>);
    fn root_element(view_state: &Self::ViewState) -> &WsElement;
}

pub struct List<C, I, VS>
where
    C: Component,
{
    parent_element: WsElement,
    template: TemplateElement,
    end_node_marker_for_partial_list: Option<web_sys::Node>,

    create_view_fn: fn(&TemplateElement, &I, &Context<C>) -> VS,
    update_view_fn: fn(&mut VS, &I, &Context<C>),
    get_view_state_root_element_fn: fn(&VS) -> &WsElement,

    items: Vec<VS>,
}

impl<C, I, VS> List<C, I, VS>
where
    I: 'static,
    C: Component + 'static,
{
    pub fn new(
        parent_element: &WsElement,
        end_node_marker_for_partial_list: Option<web_sys::Node>,
        template_string: &str,

        create_view_fn: fn(&TemplateElement, &I, &Context<C>) -> VS,
        update_view_fn: fn(&mut VS, &I, &Context<C>),
        get_view_state_root_element_fn: fn(&VS) -> &WsElement,
    ) -> Self {
        Self {
            parent_element: parent_element.clone(),
            template: TemplateElement::new(template_string),
            end_node_marker_for_partial_list,

            create_view_fn,
            update_view_fn,
            get_view_state_root_element_fn,

            items: Vec::new(),
        }
    }

    pub fn update<'a>(&mut self, item_data: impl Iterator<Item = &'a I>, context: &Context<C>) {
        let mut index = 0;
        for item_data in item_data {
            if index >= self.items.len() {
                let mut new_item = (self.create_view_fn)(&self.template, item_data, context);
                (self.update_view_fn)(&mut new_item, item_data, context);
                self.parent_element.insert_new_node_before_a_node(
                    (self.get_view_state_root_element_fn)(&new_item),
                    self.end_node_marker_for_partial_list.as_ref(),
                );
                self.items.push(new_item);
            } else {
                let old_item = unsafe { self.items.get_unchecked_mut(index) };
                (self.update_view_fn)(old_item, item_data, context);
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
                self.parent_element
                    .remove_child((self.get_view_state_root_element_fn)(&item));
            }
        }
    }
}
