use spair::{prelude::*, web_sys::MouseEvent};

use crate::AppState;

#[create_view]
impl Button {
    fn create(id: &str, text: &str, callback: CallbackArg<MouseEvent>) {}
    fn update() {}
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

#[create_view]
impl Header {
    pub fn create(comp: &Comp<AppState>) {}
    pub fn update() {}
    pub fn view() {
        div(
            class = "jumbotron",
            div(
                class = "row",
                div(class = "col-md-6", h1(text("Spair Keyed"))),
                div(
                    class = "col-md-6",
                    div(
                        class = "row",
                        Button(
                            "run",
                            "Create 1,000 rows",
                            comp.callback_arg(|state, _| state.create(1000)),
                        ),
                        Button(
                            "runlots",
                            "Create 10,000 rows",
                            comp.callback_arg(|state, _| state.create(10000)),
                        ),
                        Button(
                            "add",
                            "Append 1,000 rows",
                            comp.callback_arg(|state, _| state.append(1000)),
                        ),
                        Button(
                            "update",
                            "Update every 10th row",
                            comp.callback_arg(|state, _| state.update()),
                        ),
                        Button(
                            "clear",
                            "Clear",
                            comp.callback_arg(|state, _| state.clear()),
                        ),
                        Button(
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
