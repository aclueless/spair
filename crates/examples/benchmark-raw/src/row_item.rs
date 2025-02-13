use spairc::{prelude::Text, Context, Element, TemplateElement};

use crate::AppState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowItem {
    pub id: usize,
    pub label: String,
}

pub struct RowRender {
    key: usize,
    root_element: Element,
    _label_element: Element,
    label_text_node: Text,
    _delete_element: Element,
}

impl spairc::KeyedItemViewState<AppState> for RowRender {
    type Item = RowItem;
    type Key = usize;

    fn template_string() -> &'static str {
        pub const TEMPLATE: &str = "<tr><td class='col-md-1'>?</td><td class='col-md-4'><a class='lbl'>i</a></td><td class='col-md-1'><a class='remove'><span class='remove glyphicon glyphicon-remove' aria-hidden='true'></span></a></td><td class='col-md-6'></td></tr>";
        TEMPLATE
    }

    fn key(&self) -> &Self::Key {
        &self.key
    }

    fn key_from_item_state(state: &Self::Item) -> &Self::Key {
        &state.id
    }

    fn root_element(&self) -> &spairc::WsElement {
        &self.root_element
    }

    fn create(data: &RowItem, template: &TemplateElement, context: &Context<AppState>) -> Self {
        let element = template.create_element(0);
        let id_element = element.ws_node_ref().first_ws_element();
        let label_td = id_element.ws_node_ref().next_sibling_ws_element();
        let mut delete_element = label_td
            .ws_node_ref()
            .next_sibling_ws_element()
            .ws_node_ref()
            .first_ws_element()
            .create_element_with_capacity(1);
        let mut label_element = label_td
            .ws_node_ref()
            .first_ws_element()
            .create_element_with_capacity(1);
        id_element
            .ws_node_ref()
            .first_ws_text()
            .set_text_content(&data.id.to_string());
        let label_text_node = label_element.ws_node_ref().first_text();
        let id = data.id;
        let click_on_label = context
            .comp
            .callback_arg(move |state, _| state.set_selected_id(id));
        label_element.click(0, click_on_label);
        let click_on_delete = context
            .comp
            .callback_arg(move |state, _| state.remove_by_id(id));
        delete_element.click(0, click_on_delete);
        let mut view_state = Self {
            key: data.id,
            root_element: element,
            _label_element: label_element,
            label_text_node,
            _delete_element: delete_element,
        };

        view_state.update(&data, context);

        view_state
    }

    fn update(&mut self, data: &RowItem, context: &Context<AppState>) {
        self.label_text_node.update(&data.label);
        self.root_element
            .class_if(0, context.state.selected_id == Some(data.id), "danger");
    }
}
