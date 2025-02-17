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

impl spairc::KeyedItemView<AppState> for RowItem {
    type ViewState = RowRender;
    type Key = usize;

    fn template_string() -> &'static str {
        pub const TEMPLATE: &str = "<tr><td class='col-md-1'>?</td><td class='col-md-4'><a class='lbl'>i</a></td><td class='col-md-1'><a class='remove'><span class='remove glyphicon glyphicon-remove' aria-hidden='true'></span></a></td><td class='col-md-6'></td></tr>";
        TEMPLATE
    }

    fn get_key(&self) -> &Self::Key {
        &self.id
    }

    fn key_from_view_state(state: &Self::ViewState) -> &Self::Key {
        &state.key
    }

    fn root_element(view_state: &Self::ViewState) -> &spairc::WsElement {
        &view_state.root_element
    }

    fn create_view(
        template: &TemplateElement,
        data: &RowItem,
        context: &Context<AppState>,
    ) -> Self::ViewState {
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
        let view_state = Self::ViewState {
            key: data.id,
            root_element: element,
            _label_element: label_element,
            label_text_node,
            _delete_element: delete_element,
        };

        view_state
    }

    fn update_view(view_state: &mut Self::ViewState, data: &RowItem, context: &Context<AppState>) {
        view_state.label_text_node.update(&data.label);
        view_state
            .root_element
            .class_if(0, context.state.selected_id == Some(data.id), "danger");
    }
}
