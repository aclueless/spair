use spair::prelude::*;
use spair::{web_sys::MouseEvent, CallbackArg};

struct AppState {
    mode: Mode,
}

enum Mode {
    None,
    HtmlElement,
    View,
}

#[view]
impl Button {
    fn create_view(handler: CallbackArg<MouseEvent>, text: &str) {}
    fn update_view() {}
    fn view() {
        button(on_click = handler, text(text))
    }
}
#[component]
impl AppState {
    fn create_view(ccontext: &Context<Self>) {}
    fn update_view(ucontext: &Context<Self>) {}
    fn view() {
        div(
            replace_at_element_id = "root",
            div(
                v.Button(
                    ccontext
                        .comp
                        .callback_arg(|state, _| state.mode = Mode::None),
                    "None",
                ),
                v.Button(
                    ccontext
                        .comp
                        .callback_arg(|state, _| state.mode = Mode::HtmlElement),
                    "HTML element",
                ),
                v.Button(
                    ccontext
                        .comp
                        .callback_arg(|state, _| state.mode = Mode::View),
                    "View",
                ),
            ),
            match &ucontext.state.mode {
                Mode::None => {}
                Mode::HtmlElement => span(text("You are in Mode::HtmlElement.")),
                Mode::View => v.Button(
                    ucontext
                        .comp
                        .callback_arg(|state, _| state.mode = Mode::None),
                    "You're in Button view. Click to go back to Mode::None",
                ),
            },
        )
    }
}

fn main() {
    // wasm_logger::init(wasm_logger::Config::default());
    spair::start_app(|_| AppState { mode: Mode::None });
}
