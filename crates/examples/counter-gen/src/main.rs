use spair::{prelude::*, Text, WsElement};
use spair::{web_sys::MouseEvent, CallbackArg, Element};

struct AppState {
    value: i32,
}

pub struct UpdownButton {
    _element_1: Element,
}
impl UpdownButton {
    pub fn create_view(handler: CallbackArg<MouseEvent>, text: &str) -> Self {
        const HTML_STRING: &str = "<button>&nbsp;</button>";
        let mut _element_1 = Element::with_html(HTML_STRING, 1usize);
        _element_1.click(0usize, handler);
        let _text_2 = _element_1.ws_node_ref().first_ws_text();
        _text_2.set_text(text);
        UpdownButton { _element_1 }
    }
    pub fn update_view(&mut self) {
        let _spair_view_view_state_for_updating_ = self;
    }
    pub fn root_element(&self) -> &Element {
        &self._element_1
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
pub struct AppStateViewState {
    _element_1: Element,
    _view_2: UpdownButton,
    _text_4: Text,
    _view_5: UpdownButton,
}
impl ::spair::Component for AppState {
    type ViewState = AppStateViewState;
    fn create_view(ccontext: &Context<Self>) -> (WsElement, Self::ViewState) {
        const HTML_STRING: &str = "<div><!--view-->&nbsp;<!--view--></div>";
        let mut _element_1 = Element::with_html(HTML_STRING, 0usize);
        _element_1.replace_at_element_id("root");
        let _view_2 =
            UpdownButton::create_view(ccontext.comp.callback_arg(|state, _| state.decrease()), "-");
        let _view_marker3 = _element_1.ws_node_ref().first_ws_node();
        _element_1.insert_new_node_before_a_node(_view_2.root_element(), Some(&_view_marker3));
        let _text_4 = _view_marker3.ws_node_ref().next_sibling_text();
        let _view_5 =
            UpdownButton::create_view(ccontext.comp.callback_arg(|state, _| state.increase()), "+");
        let _view_marker6 = _text_4.ws_node_ref().next_sibling_ws_node();
        _element_1.insert_new_node_before_a_node(_view_5.root_element(), Some(&_view_marker6));
        (
            _element_1.ws_element().clone(),
            AppStateViewState {
                _element_1,
                _view_2,
                _text_4,
                _view_5,
            },
        )
    }
    fn update_view(
        _spair_component_view_state_for_updating_: &mut Self::ViewState,
        ucontext: &Context<Self>,
    ) {
        _spair_component_view_state_for_updating_
            ._view_2
            .update_view();
        _spair_component_view_state_for_updating_
            ._text_4
            .update(ucontext.state.value);
        _spair_component_view_state_for_updating_
            ._view_5
            .update_view();
    }
}

fn main() {
    // wasm_logger::init(wasm_logger::Config::default());
    spair::start_app(|_| AppState { value: 42 });
}
