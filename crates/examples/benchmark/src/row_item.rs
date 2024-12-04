use spairc::{Context, Element, TemplateElement};

use crate::AppState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowData {
    pub id: usize,
    pub label: String,
}

pub struct RowRender {
    key: usize,
    root_element: Element,
    label_element: Element,
    label_string: String,
    _delete_element: Element,
}

impl spairc::KeyedItemRender<AppState> for RowRender {
    type Item = RowData;
    type Key = usize;

    fn template_string() -> &'static str {
        pub const TEMPLATE: &str = "<tr><td class='col-md-1'></td><td class='col-md-4'><a class='lbl'></a></td><td class='col-md-1'><a class='remove'><span class='remove glyphicon glyphicon-remove' aria-hidden='true'></span></a></td><td class='col-md-6'></td></tr>";
        TEMPLATE
    }

    fn key(&self) -> &Self::Key {
        &self.key
    }

    fn key_from_state(state: &Self::Item) -> &Self::Key {
        &state.id
    }

    fn root_element(&self) -> &spairc::WsElement {
        &self.root_element
    }

    fn create(data: &RowData, template: &TemplateElement, context: &Context<AppState>) -> Self {
        let element = template.create_element(0);
        let id_element = element.first_child();
        let label_td = id_element.next_sibling();
        let mut delete_element = label_td
            .next_sibling()
            .first_child()
            .create_element_with_capacity(1);
        let mut label_element = label_td.first_child().create_element_with_capacity(1);
        id_element.set_text_content(&data.id.to_string());
        label_element.set_text_content(&data.label);
        let id = data.id;
        let click_on_label = context
            .comp
            .callback_arg(move |state, _| state.set_selected_id(id));
        label_element.click(0, click_on_label);
        let click_on_delete = context
            .comp
            .callback_arg(move |state, _| state.remove_by_id(id));
        delete_element.click(0, click_on_delete);
        Self {
            key: data.id,
            root_element: element,
            label_element,
            label_string: data.label.clone(),
            _delete_element: delete_element,
        }
    }

    fn update(&mut self, data: &RowData, context: &Context<AppState>) {
        self.label_element
            .update_text_content_with_str(&mut self.label_string, &data.label);
        self.root_element
            .class_if(0, context.state.selected_id == Some(data.id), "danger");
    }
}
