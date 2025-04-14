use spair::{prelude::*, web_sys::MouseEvent};

use crate::AppState;

#[new_view]
impl Button {
    fn create_view(id: &str, text: &str, callback: CallbackArg<MouseEvent>) {}
    fn update_view() {}
    fn view() {
        div(
            class = "col-sm-6 smallpad",
            button(
                id = id,
                class = "btn btn-primary btn-block",
                r#type = "button",
                on_click = callback,
                text(text),
            ),
        )
    }
}

#[new_view]
impl Header {
    fn create_view(comp: &Comp<AppState>) {}
    fn update_view() {}
    fn view() {
        div(
            class = "jumbotron",
            div(
                class = "row",
                div(class = "col-md-6", h1(text("Spair Keyed"))),
                div(
                    class = "col-md-6",
                    div(
                        class = "row",
                        v.Button(
                            "run",
                            "Create 1,000 rows",
                            comp.callback_arg(|state, _| state.create(1000)),
                        ),
                        v.Button(
                            "runlots",
                            "Create 10,000 rows",
                            comp.callback_arg(|state, _| state.create(10000)),
                        ),
                        v.Button(
                            "add",
                            "Append 1,000 rows",
                            comp.callback_arg(|state, _| state.append(1000)),
                        ),
                        v.Button(
                            "update",
                            "Update every 10th row",
                            comp.callback_arg(|state, _| state.update()),
                        ),
                        v.Button(
                            "clear",
                            "Clear",
                            comp.callback_arg(|state, _| state.clear()),
                        ),
                        v.Button(
                            "swaprows",
                            "Swap Rows",
                            comp.callback_arg(|state, _| state.swap()),
                        ),
                    ),
                ),
            ),
        )
    }
}
