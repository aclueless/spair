use spair::prelude::*;

use crate::AppState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowItem {
    pub id: usize,
    pub label: String,
}
#[keyed_list_item_for]
impl RowItem {
    fn get_key(&self) -> &usize {
        &self.id
    }
    fn create_view(cdata: &Self, ccontext: &Context<AppState>) {
        let id = cdata.id;
    }
    fn update_view(udata: &Self, ucontext: &Context<AppState>) {}
    fn view() {
        tr(
            class_if = (Some(udata.id) == ucontext.state.selected_id, "danger"),
            td(class = "col-md-1", text(cdata.id)),
            td(
                class = "col-md-4",
                a(
                    on_click = ccontext
                        .comp
                        .callback_arg(move |state, _| state.set_selected_id(id)),
                    text(udata.label.as_str()),
                ),
            ),
            td(
                class = "col-md-1",
                a(
                    on_click = ccontext
                        .comp
                        .callback_arg(move |state, _| state.remove_by_id(id)),
                    span(class = "glyphicon glyphicon-remove", aria_hidden = true),
                ),
            ),
            td(class = "col-md-6"),
        )
    }
}
