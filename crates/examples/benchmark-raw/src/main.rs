use header::HeaderViewState;
use row_item::RowItem;
use spairc::{Component, Context, Element, KeyedList};

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
    keyed_list: KeyedList<AppState, row_item::RowRender>,
}

impl Component for AppState {
    type ViewState = AppViewState;

    fn init(&self, comp: &spairc::Comp<Self>) -> (spairc::ComponentRoot, Self::ViewState) {
        const HTML: &str = "<div id='main'><div class='container'><table class='table table-hover table-striped test-data'><tbody id='tbody'></tbody></table><span class='preloadicon glyphicon glyphicon-remove' aria-hidden='true'></span></div></div>";
        let context = Context { comp, state: self };
        //
        let _root_element = Element::with_html(HTML, 0);
        let container = _root_element.first_child();
        let table_element = container.first_child();
        let _header = HeaderViewState::create(&context);
        container.insert_new_node_before_a_node(&_header.root_element, Some(&table_element));
        let tbody = table_element.first_child();
        let mut keyed_list = KeyedList::new(tbody);
        keyed_list.update(self.rows.iter(), Context { comp, state: self });
        _root_element.append_to_body();
        let updater = AppViewState {
            _root_element,
            _header,
            keyed_list,
        };
        (spairc::ComponentRoot::Body, updater)
    }

    fn render(&self, updater: &mut Self::ViewState, comp: &spairc::Comp<Self>) {
        log::info!("running update");
        updater
            .keyed_list
            .update(self.rows.iter(), Context { comp, state: self });
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
    spairc::start_app(AppState {
        next_id: 1,
        rows: Vec::new(),
        selected_id: None,
    });
}
