use spair::prelude::*;

use crate::AppState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowItem {
    pub id: usize,
    pub label: String,
}

#[new_view]
impl Table {
    fn create() {}
    fn update(app_state: &AppState, context: &Context<AppState>) {}
    fn view() {
        table(
            class = "table table-hover table-striped test-data",
            tbody(
                inlined.list(
                    context,
                    app_state.rows.iter(),
                    |item| -> &usize { &item.id },
                    |citem, ccontext| {
                        let id = citem.id;
                    },
                    |uitem, ucontext| {},
                    tr(
                        class_if = (Some(uitem.id) == ucontext.state.selected_id, "danger"),
                        td(class = "col-md-1", text(citem.id)),
                        td(
                            class = "col-md-4",
                            a(
                                on_click = ccontext
                                    .comp
                                    .callback_arg(move |state, _| state.set_selected_id(id)),
                                text(uitem.label.as_str()),
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
                    ),
                ),
            ),
        )
    }
}
