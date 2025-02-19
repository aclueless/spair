use header::Header;
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

#[component(?)]
impl AppState {
    fn create_view(_cdata: &Self, ccomp: &Comp<Self>) {}
    fn update_view(udata: &Self, ucomp: &Comp<Self>) {}
    fn view() {
        div(
            replace_at_element_id = "main",
            div(
                class = "container",
                view::Header(create_view(&ccomp), update_view()),
                table(
                    class = "table table-hover table-striped test-data",
                    tbody(list_of(
                        AppState,
                        RowItem,
                        ucomp.context(udata),
                        udata.rows.iter(),
                    )),
                ),
                span(
                    class = "preloadicon glyphicon glyphicon-remove",
                    aria_hidden = true,
                ),
            ),
        )
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
