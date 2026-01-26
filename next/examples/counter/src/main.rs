use spair::prelude::*;
use spair::{CallbackArg, web_sys::MouseEvent};

struct AppState {
    value: i32,
}

#[create_view]
impl UpdownButton {
    fn create(handler: CallbackArg<MouseEvent>, text: &str) {}
    fn update() {}
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
#[impl_component]
impl AppState {
    fn create(ccontext: &Context<Self>) {}
    fn update(ucontext: &Context<Self>) {}
    fn view() {
        div(
            replace_at_element_id = "root",
            UpdownButton(ccontext.comp.callback_arg(|state, _| state.decrease()), "-"),
            ucontext.state.value,
            UpdownButton(ccontext.comp.callback_arg(|state, _| state.increase()), "+"),
        )
    }
}

fn main() {
    // wasm_logger::init(wasm_logger::Config::default());
    spair::start_app(|_| AppState { value: 42 });
}
