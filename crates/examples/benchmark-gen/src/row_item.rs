#![allow(clippy::all)]
use spair::{Element, Text, WsElement, prelude::*};

use crate::AppState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowItem {
    pub id: usize,
    pub label: String,
}
pub struct RowItemViewState {
    key: usize,
    _element_1: Element,
    _element_5: Element,
    _text_6: Text,
    _element_8: Element,
}
impl ::spair::KeyedListItemView<AppState> for RowItem {
    type ViewState = RowItemViewState;
    type Key = usize;
    fn get_key(&self) -> &usize {
        &self.id
    }
    fn key_from_view_state(view_state: &Self::ViewState) -> &Self::Key {
        &view_state.key
    }
    fn root_element(view_state: &Self::ViewState) -> &WsElement {
        &view_state._element_1
    }
    fn template_string() -> &'static str {
        "<tr><td class='col-md-1'>&nbsp;</td><td class='col-md-4'><a>&nbsp;</a></td><td class='col-md-1'><a><span class='glyphicon glyphicon-remove' aria-hidden='true'></span></a></td><td class='col-md-6'></td></tr>"
    }
    fn create_view(
        _keyed_view_state_template: &::spair::TemplateElement,
        cdata: &Self,
        ccontext: &Context<AppState>,
    ) -> Self::ViewState {
        let id = cdata.id;
        let _element_1 = _keyed_view_state_template.create_element(1usize);
        let _element_2 = _element_1.ws_node_ref().first_ws_element();
        let _text_3 = _element_2.ws_node_ref().first_ws_text();
        _text_3.set_text(cdata.id);
        let _element_4 = _element_2.ws_node_ref().next_sibling_ws_element();
        let _element_5 = _element_4.ws_node_ref().first_ws_element();
        let mut _element_5 = _element_5.create_element_with_capacity(1usize);
        _element_5.click(
            0usize,
            ccontext
                .comp
                .callback_arg(move |state, _| state.set_selected_id(id)),
        );
        let _text_6 = _element_5.ws_node_ref().first_text();
        let _element_7 = _element_4.ws_node_ref().next_sibling_ws_element();
        let _element_8 = _element_7.ws_node_ref().first_ws_element();
        let mut _element_8 = _element_8.create_element_with_capacity(1usize);
        _element_8.click(
            0usize,
            ccontext
                .comp
                .callback_arg(move |state, _| state.remove_by_id(id)),
        );
        let _element_9 = _element_8.ws_node_ref().first_ws_element();
        let _element_10 = _element_7.ws_node_ref().next_sibling_ws_element();
        RowItemViewState {
            key: cdata.get_key().clone(),
            _element_1,
            _element_5,
            _text_6,
            _element_8,
        }
    }
    fn update_view(
        _spair_keyed_item_view_state_for_updating_: &mut Self::ViewState,
        udata: &Self,
        ucontext: &Context<AppState>,
    ) {
        _spair_keyed_item_view_state_for_updating_
            ._element_1
            .class_if_with_index(
                0usize,
                Some(udata.id) == ucontext.state.selected_id,
                "danger",
            );
        _spair_keyed_item_view_state_for_updating_
            ._text_6
            .update(udata.label.as_str());
    }
}
