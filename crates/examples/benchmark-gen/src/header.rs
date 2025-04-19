use spair::{Element, prelude::*, web_sys::MouseEvent};

use crate::AppState;

pub struct Button {
    _element_1: Element,
    _element_2: Element,
}
impl Button {
    pub fn create_view(id: &str, text: &str, callback: CallbackArg<MouseEvent>) -> Self {
        const HTML_STRING: &str = "<div class='col-sm-6 smallpad'><button class='btn btn-primary btn-block' r#type='button'>&nbsp;</button></div>";
        let mut _element_1 = Element::with_html(HTML_STRING, 0usize);
        let _element_2 = _element_1.ws_node_ref().first_ws_element();
        let mut _element_2 = _element_2.create_element_with_capacity(1usize);
        _element_2.set_id(id);
        _element_2.click(0usize, callback);
        let _text_3 = _element_2.ws_node_ref().first_ws_text();
        _text_3.set_text(text);
        Button {
            _element_1,
            _element_2,
        }
    }
    pub fn update_view(&mut self) {
        let _spair_view_view_state_for_updating_ = self;
    }
    pub fn root_element(&self) -> &Element {
        &self._element_1
    }
}
pub struct Header {
    _element_1: Element,
    _view_8: Button,
    _view_10: Button,
    _view_12: Button,
    _view_14: Button,
    _view_16: Button,
    _view_18: Button,
}
impl Header {
    pub fn create_view(comp: &Comp<AppState>) -> Self {
        const HTML_STRING: &str = "<div class='jumbotron'><div class='row'><div class='col-md-6'><h1>Spair Keyed</h1></div><div class='col-md-6'><div class='row'><!--view--><!--view--><!--view--><!--view--><!--view--><!--view--></div></div></div></div>";
        let mut _element_1 = Element::with_html(HTML_STRING, 0usize);
        let _element_2 = _element_1.ws_node_ref().first_ws_element();
        let _element_3 = _element_2.ws_node_ref().first_ws_element();
        let _element_4 = _element_3.ws_node_ref().first_ws_element();
        let _text_5 = _element_4.ws_node_ref().first_ws_text();
        let _element_6 = _element_3.ws_node_ref().next_sibling_ws_element();
        let _element_7 = _element_6.ws_node_ref().first_ws_element();
        let _view_8 = Button::create_view(
            "run",
            "Create 1,000 rows",
            comp.callback_arg(|state, _| state.create(1000)),
        );
        let _view_marker9 = _element_7.ws_node_ref().first_ws_node();
        _element_7.insert_new_node_before_a_node(_view_8.root_element(), Some(&_view_marker9));
        let _view_10 = Button::create_view(
            "runlots",
            "Create 10,000 rows",
            comp.callback_arg(|state, _| state.create(10000)),
        );
        let _view_marker11 = _view_marker9.ws_node_ref().next_sibling_ws_node();
        _element_7.insert_new_node_before_a_node(_view_10.root_element(), Some(&_view_marker11));
        let _view_12 = Button::create_view(
            "add",
            "Append 1,000 rows",
            comp.callback_arg(|state, _| state.append(1000)),
        );
        let _view_marker13 = _view_marker11.ws_node_ref().next_sibling_ws_node();
        _element_7.insert_new_node_before_a_node(_view_12.root_element(), Some(&_view_marker13));
        let _view_14 = Button::create_view(
            "update",
            "Update every 10th row",
            comp.callback_arg(|state, _| state.update()),
        );
        let _view_marker15 = _view_marker13.ws_node_ref().next_sibling_ws_node();
        _element_7.insert_new_node_before_a_node(_view_14.root_element(), Some(&_view_marker15));
        let _view_16 = Button::create_view(
            "clear",
            "Clear",
            comp.callback_arg(|state, _| state.clear()),
        );
        let _view_marker17 = _view_marker15.ws_node_ref().next_sibling_ws_node();
        _element_7.insert_new_node_before_a_node(_view_16.root_element(), Some(&_view_marker17));
        let _view_18 = Button::create_view(
            "swaprows",
            "Swap Rows",
            comp.callback_arg(|state, _| state.swap()),
        );
        let _view_marker19 = _view_marker17.ws_node_ref().next_sibling_ws_node();
        _element_7.insert_new_node_before_a_node(_view_18.root_element(), Some(&_view_marker19));
        Header {
            _element_1,
            _view_8,
            _view_10,
            _view_12,
            _view_14,
            _view_16,
            _view_18,
        }
    }
    pub fn update_view(&mut self) {
        let _spair_view_view_state_for_updating_ = self;
        _spair_view_view_state_for_updating_._view_8.update_view();
        _spair_view_view_state_for_updating_._view_10.update_view();
        _spair_view_view_state_for_updating_._view_12.update_view();
        _spair_view_view_state_for_updating_._view_14.update_view();
        _spair_view_view_state_for_updating_._view_16.update_view();
        _spair_view_view_state_for_updating_._view_18.update_view();
    }
    pub fn root_element(&self) -> &Element {
        &self._element_1
    }
}
