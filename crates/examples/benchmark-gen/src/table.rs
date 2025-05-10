#![allow(clippy::all)]
use spair::prelude::*;

use crate::AppState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowItem {
    pub id: usize,
    pub label: String,
}

pub struct Table {
    _element_table_1: ::spair::Element,
    _ilist13: ::spair::KeyedList<usize, _InlinedListItemViewState15>,
}
struct _InlinedListItemViewState15 {
    _element_tr_3: ::spair::Element,
    _element_a_7: ::spair::Element,
    _text_8: ::spair::Text,
    _element_a_10: ::spair::Element,
}
impl _InlinedListItemViewState15 {
    fn root_element(&self) -> &::spair::WsElement {
        &self._element_tr_3
    }
}
impl Table {
    pub fn create() -> Self {
        const HTML_STRING: &str =
            "<table class='table table-hover table-striped test-data'><tbody></tbody></table>";
        let mut _element_table_1 = ::spair::Element::with_html(HTML_STRING, 0usize);
        let _element_tbody_2 = _element_table_1.ws_node_ref().first_ws_element();
        let _ilist_end_flag14 = None;
        let _ilist13 = ::spair::KeyedList::new(
            &_element_tbody_2,
            _ilist_end_flag14.clone(),
            "<tr><td class='col-md-1'>&nbsp;</td><td class='col-md-4'><a>&nbsp;</a></td><td class='col-md-1'><a><span class='glyphicon glyphicon-remove' aria-hidden='true'></span></a></td><td class='col-md-6'></td></tr>",
        );
        Table {
            _element_table_1,
            _ilist13,
        }
    }
    pub fn update(&mut self, app_state: &AppState, context: &Context<AppState>) {
        let _spair_view_view_state_for_updating_ = self;
        _spair_view_view_state_for_updating_._ilist13.update(app_state.rows.iter(),
        context, | item | -> & usize { & item.id }, |
        _keyed_view_state_template : & :: spair :: TemplateElement, citem,
        ccontext |
        {
            let id = citem.id; const HTML_STRING : & str =
            "<tr><td class='col-md-1'>&nbsp;</td><td class='col-md-4'><a>&nbsp;</a></td><td class='col-md-1'><a><span class='glyphicon glyphicon-remove' aria-hidden='true'></span></a></td><td class='col-md-6'></td></tr>";
            let mut _element_tr_3 = :: spair :: Element ::
            with_html(HTML_STRING, 1usize); let _element_td_4 =
            _element_tr_3.ws_node_ref().first_ws_element(); let _text_5 =
            _element_td_4.ws_node_ref().first_ws_text();
            _text_5.set_text(citem.id); let _element_td_6 =
            _element_td_4.ws_node_ref().next_sibling_ws_element(); let
            _element_a_7 = _element_td_6.ws_node_ref().first_ws_element(); let
            mut _element_a_7 =
            _element_a_7.create_element_with_capacity(1usize);
            _element_a_7.click(0usize,
            ccontext.comp.callback_arg(move | state, _ |
            state.set_selected_id(id))); let _text_8 =
            _element_a_7.ws_node_ref().first_text(); let _element_td_9 =
            _element_td_6.ws_node_ref().next_sibling_ws_element(); let
            _element_a_10 = _element_td_9.ws_node_ref().first_ws_element();
            let mut _element_a_10 =
            _element_a_10.create_element_with_capacity(1usize);
            _element_a_10.click(0usize,
            ccontext.comp.callback_arg(move | state, _ |
            state.remove_by_id(id))); let _element_span_11 =
            _element_a_10.ws_node_ref().first_ws_element(); let _element_td_12
            = _element_td_9.ws_node_ref().next_sibling_ws_element();
            _InlinedListItemViewState15
            { _element_tr_3, _element_a_7, _text_8, _element_a_10, }
        }, | _inlined_list_item_view_state_ : & mut
        _InlinedListItemViewState15, uitem, ucontext |
        {
            _inlined_list_item_view_state_._element_tr_3.class_if_with_index(0usize,
            Some(uitem.id) == ucontext.state.selected_id, "danger");
            _inlined_list_item_view_state_._text_8.update(uitem.label.as_str());
        }, _InlinedListItemViewState15 :: root_element);
    }
    pub fn root_element(&self) -> &::spair::Element {
        &self._element_table_1
    }
}
