use header::HeaderViewState;
use row_item::RowItem;
use spairc::prelude::*;
use spairc::{Component, Element, KeyedList};

mod header;
mod row_item;

struct AppState {
    next_id: usize,
    rows: Vec<RowItem>,
    selected_id: Option<usize>,
}

struct AppViewState {
    _root_element: Element,
    _header: HeaderViewState,
    keyed_list: KeyedList<AppState, row_item::RowItem>,
}

impl Component for AppState {
    type ViewState = AppViewState;

    fn create_view(cstate: &Self, ccomp: &Comp<Self>) -> (spairc::WsElement, Self::ViewState) {
        const HTML: &str = "<div id='main'><div class='container'><table class='table table-hover table-striped test-data'><tbody id='tbody'></tbody></table><span class='preloadicon glyphicon glyphicon-remove' aria-hidden='true'></span></div></div>";
        let _root_element = Element::with_html(HTML, 0);
        let container = _root_element.ws_node_ref().first_ws_element();
        let table_element = container.ws_node_ref().first_ws_element();
        log::info!("before header");
        let context = ccomp.context(cstate);
        let _header = HeaderViewState::create(&context);
        container.insert_new_node_before_a_node(&_header.root_element, Some(&table_element));
        let tbody = table_element.ws_node_ref().first_ws_element();
        log::info!("before list");
        let mut keyed_list = KeyedList::new(tbody);
        keyed_list.update(cstate.rows.iter(), context);
        _root_element.append_to_body();
        let view_state = AppViewState {
            _root_element,
            _header,
            keyed_list,
        };
        (view_state._root_element.clone(), view_state)
    }

    fn update_view(view_state: &mut Self::ViewState, ustate: &Self, ucomp: &Comp<Self>) {
        log::info!("running update");
        view_state
            .keyed_list
            .update(ustate.rows.iter(), ucomp.context(ustate));
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
    let index = (spairc::web_sys::js_sys::Math::random() * 1000.0) as usize % item_count;
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
    spairc::start_app(|_| AppState {
        next_id: 1,
        rows: Vec::new(),
        selected_id: None,
    });
}
