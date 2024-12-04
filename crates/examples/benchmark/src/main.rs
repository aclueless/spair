use row_item::RowData;
use spairc::{CallbackArg, Comp, Component, Context, Element, KeyedList, WsElement};

mod row_item;

struct AppState {
    todos: Vec<row_item::RowData>,
    selected_id: Option<usize>,
}

struct AppViewUpdater {
    table_element: Element,
    keyed_list: KeyedList<AppState, row_item::RowRender>,
}

impl Component for AppState {
    type Updater = AppViewUpdater;

    fn init(&self, comp: &spairc::Comp<Self>) -> (spairc::ComponentRoot, Self::Updater) {
        let table_element = Element::new("<table></table>", 0);
        let mut keyed_list = KeyedList::new(table_element.ws_element().clone());
        keyed_list.update(self.todos.iter(), Context { comp, state: self });
        table_element.append_to_body();
        let updater = AppViewUpdater {
            table_element,
            keyed_list,
        };
        (spairc::ComponentRoot::Body, updater)
    }

    fn render(&self, updater: &mut Self::Updater, comp: &spairc::Comp<Self>) {
        log::info!("running update");
        updater
            .keyed_list
            .update(self.todos.iter(), Context { comp, state: self });
    }
}

impl AppState {
    fn set_selected_id(&mut self, id: usize) {}
    fn remove_by_id(&mut self, id: usize) {}
}

struct AppComp {
    comp: Comp<AppState>,
}

impl AppComp {
    fn set_selected_id(&self) -> CallbackArg<usize> {
        self.comp.callback_arg(AppState::set_selected_id)
    }
    fn remove_by_id(&self) -> CallbackArg<usize> {
        self.comp.callback_arg(AppState::remove_by_id)
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    spairc::start_app(AppState {
        todos: vec![RowData {
            id: 1,
            label: "test".to_string(),
        }],
        selected_id: None,
    });
}
