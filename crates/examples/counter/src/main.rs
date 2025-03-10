use spairc::prelude::*;
use spairc::{web_sys::MouseEvent, CallbackArg, Element};

struct AppState {
    value: i32,
}

#[view]
impl UpdownButton {
    fn create_view(handler: CallbackArg<MouseEvent>, text: &str) {}
    fn update_view() {}
    fn view() {
        button(on_click = handler, text(text))
    }
}

impl AppState {
    fn increase(&mut self) {
        self.value += 1;
    }

    fn decrease(&mut self) {
        self.value -= 1;
    }
}
#[component]
impl AppState {
    fn create_view(ccontext: &Context<Self>) {}
    fn update_view(ucontext: &Context<Self>) {}
    fn view() {
        div(
            replace_at_element_id = "root",
            v.UpdownButton(ccontext.comp.callback_arg(|state, _| state.decrease()), "-"),
            text(ucontext.state.value),
            v.UpdownButton(ccontext.comp.callback_arg(|state, _| state.increase()), "+"),
        )
    }
}

fn main() {
    // wasm_logger::init(wasm_logger::Config::default());
    spairc::start_app(|_| AppState { value: 42 });
}
