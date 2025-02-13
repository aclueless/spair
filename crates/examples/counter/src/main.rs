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
        button(click = handler, text(text))
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
    fn create_view(_cstate: &Self, ccomp: &Comp<Self>) {}
    fn update_view(ustate: &Self, _ucomp: &Comp<Self>) {}
    fn view() {
        div(
            replace_at_element_id = "root",
            view::UpdownButton(
                create_view(ccomp.callback_arg(|state, _| state.decrease()), "-"),
                update_view(),
            ),
            text(ustate.value),
            view::UpdownButton(
                create_view(ccomp.callback_arg(|state, _| state.increase()), "+"),
                update_view(),
            ),
        )
    }
}

fn main() {
    // wasm_logger::init(wasm_logger::Config::default());
    spairc::start_app(|_| AppState { value: 42 });
}
