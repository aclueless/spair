use header::Header;
use row_item::RowItem;
use spair::prelude::*;
use spair::{Component, Element, KeyedList};

mod header;
mod row_item;

struct AppState {
    next_id: usize,
    rows: Vec<RowItem>,
    selected_id: Option<usize>,
}
pub struct AppStateViewState {
    _element_1: Element,
    _view_3: Header,
    _keyed_list7: ::spair::KeyedList<AppState, RowItem>,
}
impl Component for AppState {
    type ViewState = AppStateViewState;
    fn create_view(ccontext: &Context<Self>) -> (::spair::WsElement, Self::ViewState) {
        const HTML_STRING: &str = "<div><div class='container'><!--view--><table class='table table-hover table-striped test-data'><tbody></tbody></table><span class='preloadicon glyphicon glyphicon-remove' aria-hidden='true'></span></div></div>";
        let mut _element_1 = Element::with_html(HTML_STRING, 0usize);
        _element_1.replace_at_element_id("main");
        let _element_2 = _element_1.ws_node_ref().first_ws_element();
        let _view_3 = Header::create_view(ccontext.comp);
        let _view_marker4 = _element_2.ws_node_ref().first_ws_node();
        _element_2.insert_new_node_before_a_node(_view_3.root_element(), Some(&_view_marker4));
        let _element_5 = _view_marker4.ws_node_ref().next_sibling_ws_element();
        let _element_6 = _element_5.ws_node_ref().first_ws_element();
        let _keyed_list_end_flag8 = None;
        let _keyed_list7 = KeyedList::new(&_element_6, _keyed_list_end_flag8.clone());
        let _element_9 = _element_5.ws_node_ref().next_sibling_ws_element();
        (
            _element_1.ws_element().clone(),
            AppStateViewState {
                _element_1,
                _view_3,
                _keyed_list7,
            },
        )
    }
    fn update_view(
        _spair_component_view_state_for_updating_: &mut Self::ViewState,
        ucontext: &Context<Self>,
    ) {
        _spair_component_view_state_for_updating_
            ._view_3
            .update_view();
        _spair_component_view_state_for_updating_
            ._keyed_list7
            .update(ucontext.state.rows.iter(), ucontext);
    }
}

impl AppState {
    fn set_selected_id(&mut self, id: usize) {
        self.selected_id = Some(id);
    }

    fn remove_by_id(&mut self, id: usize) {
        self.rows.retain(|row| row.id != id);
    }

    fn clear(&mut self) {
        self.rows.clear();
        self.next_id = 1;
    }

    fn create(&mut self, count: usize) {
        self.clear();
        self.append(count);
    }

    fn append(&mut self, count: usize) {
        self.rows.reserve_exact(count);

        for _ in 0..count {
            let adjective = select_random(ADJECTIVES);
            let colour = select_random(COLOURS);
            let noun = select_random(NOUNS);
            let capacity = adjective.len() + colour.len() + noun.len() + 2;
            let mut label = String::with_capacity(capacity);
            label.push_str(adjective);
            label.push(' ');
            label.push_str(colour);
            label.push(' ');
            label.push_str(noun);

            self.rows.push(RowItem {
                id: self.next_id,
                label,
            });
            self.next_id += 1;
        }
    }

    fn update(&mut self) {
        for row in self.rows.iter_mut().step_by(10) {
            row.label += " !!!";
        }
    }

    fn swap(&mut self) {
        if self.rows.len() > 998 {
            self.rows.swap(1, 998);
        }
    }
}

fn select_random<'a>(data: &[&'a str]) -> &'a str {
    let item_count = data.len();
    let index = (spair::web_sys::js_sys::Math::random() * 1000.0) as usize % item_count;
    data[index]
}

static ADJECTIVES: &[&str] = &[
    "pretty",
    "large",
    "big",
    "small",
    "tall",
    "short",
    "long",
    "handsome",
    "plain",
    "quaint",
    "clean",
    "elegant",
    "easy",
    "angry",
    "crazy",
    "helpful",
    "mushy",
    "odd",
    "unsightly",
    "adorable",
    "important",
    "inexpensive",
    "cheap",
    "expensive",
    "fancy",
];

static COLOURS: &[&str] = &[
    "red", "yellow", "blue", "green", "pink", "brown", "purple", "brown", "white", "black",
    "orange",
];

static NOUNS: &[&str] = &[
    "table", "chair", "house", "bbq", "desk", "car", "pony", "cookie", "sandwich", "burger",
    "pizza", "mouse", "keyboard",
];

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    spair::start_app(|_| AppState {
        next_id: 1,
        rows: Vec::new(),
        selected_id: None,
    });
}
