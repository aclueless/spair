use std::ops::Not;

use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use spair::{
    prelude::*,
    web_sys::{EventTarget, FocusEvent, HtmlInputElement, KeyboardEvent},
};
use wasm_bindgen::{JsCast, UnwrapThrowExt};

#[derive(Default, Serialize, Deserialize)]
struct TodoList {
    next_id: u32,
    items: Vec<TodoItem>,
}

#[derive(Serialize, Deserialize)]
struct TodoItem {
    id: u32,
    title: String,
    completed: bool,
}

struct App {
    data: TodoList,

    filter: Filter,
    editing_id: Option<u32>,
    new_todo_title: String,
}

#[derive(PartialEq, Clone, Copy)]
enum Filter {
    All,
    Active,
    Completed,
}

impl TodoItem {
    fn visible(&self, filter: &Filter) -> bool {
        matches!(
            (filter, self.completed),
            (Filter::All, _) | (Filter::Active, false) | (Filter::Completed, true)
        )
    }
}

impl App {
    fn set_filter(&mut self, filter: Filter) {
        self.filter = filter;
    }

    fn set_new_todo_title(&mut self, value: String) {
        self.new_todo_title = value;
    }

    fn create_new_todo(&mut self) {
        let title = std::mem::take(&mut self.new_todo_title).trim().to_string();
        if title.is_empty() {
            return;
        }
        self.data.items.push(TodoItem {
            id: self.data.next_id,
            title,
            completed: false,
        });
        self.data.next_id += 1;
        write_data_to_storage(&self.data);
    }

    fn visible_items(&self) -> impl Iterator<Item = &TodoItem> {
        self.data
            .items
            .iter()
            .filter(|item| item.visible(&self.filter))
    }

    fn is_all_completed(&self) -> bool {
        self.data.items.iter().all(|item| item.completed)
    }

    fn toggle_all(&mut self) {
        let checked = !self.is_all_completed();
        self.data
            .items
            .iter_mut()
            .for_each(|item| item.completed = checked);
        write_data_to_storage(&self.data);
    }

    fn toggle(&mut self, id: u32) {
        if let Some(item) = self.data.items.iter_mut().find(|item| item.id == id) {
            item.completed = !item.completed;
            write_data_to_storage(&self.data);
        }
    }

    fn clear_completed(&mut self) {
        self.data.items.retain(|item| !item.completed);
        write_data_to_storage(&self.data);
    }

    fn remove(&mut self, id: u32) {
        self.data.items.retain(|item| item.id != id);
        write_data_to_storage(&self.data);
    }

    fn start_editing(&mut self, id: u32) {
        self.editing_id = Some(id);
    }

    fn end_editing(&mut self, target: Option<EventTarget>) {
        let id = match self.editing_id {
            Some(id) => id,
            None => return,
        };
        match target.and_then(|v| {
            let title = v.unchecked_into::<HtmlInputElement>().value();
            let title = title.trim();
            if title.is_empty() {
                None
            } else {
                Some(title.to_string())
            }
        }) {
            Some(title) => {
                self.data
                    .items
                    .iter_mut()
                    .find(|item| item.id == id)
                    .expect_throw("Why editing item with an invalid id?")
                    .title = title.to_string();
                write_data_to_storage(&self.data);
            }
            None => self.remove(id),
        }
        self.editing_id = None;
    }

    fn cancel_editing(&mut self) {
        self.editing_id = None;
    }
}

const TODO_DATA_KEY: &str = "spair-todos-list";

pub(crate) fn write_data_to_storage(data: &TodoList) {
    LocalStorage::set(TODO_DATA_KEY, data).expect_throw("Unable to set item on local storage")
}

pub(crate) fn read_data_from_storage() -> TodoList {
    LocalStorage::get(TODO_DATA_KEY).unwrap_or_default()
}

impl spair::Route for Filter {
    fn from_location(location: &spair::web_sys::Location) -> Self {
        if let Ok(hash) = location.hash() {
            match hash.as_str() {
                "#/all" => Self::All,
                "#/active" => Self::Active,
                "#/completed" => Self::Completed,
                _ => Self::All,
            }
        } else {
            Self::All
        }
    }

    fn url(&self) -> String {
        match self {
            Filter::All => "#/all",
            Filter::Active => "#/active",
            Filter::Completed => "#/completed",
        }
        .to_string()
    }
}

#[component_for]
impl App {
    fn create(cc: &Context<Self>) {}
    fn update(uc: &Context<Self>) {}
    fn view() {
        section(
            replace_at_element_id = "root",
            class = "todoapp",
            v.Header(cc.comp).update(&uc.state.new_todo_title),
            section(
                class = "main",
                class_if = (uc.state.data.items.is_empty(), "hidden"),
                input(
                    id = "toggle-all",
                    class = "toggle-all",
                    r#type = "checkbox",
                    checked = uc.state.is_all_completed(),
                    on_change = cc.comp.callback_arg(|state, _| state.toggle_all()),
                ),
                label(r#for = "toggle-all", text("Mark all as complete")),
                ul(
                    class = "todo-list",
                    kl.TodoItem.App(uc, uc.state.visible_items()),
                ),
            ),
            v.Footer(cc.comp).update(uc.state),
        )
    }
}

const ESCAPE_KEY: &str = "Escape";
const ENTER_KEY: &str = "Enter";

#[new_view]
impl Header {
    fn create(ccomp: &Comp<App>) {}
    fn update(new_todo_title: &str) {}
    fn view() {
        header(
            class = "header",
            h1(text("Spair Todos")),
            input(
                class = "new-todo",
                autofocus = true,
                placeholder = "What needs to be done?",
                value = new_todo_title,
                on_keydown = ccomp.callback_arg(|state, event: KeyboardEvent| {
                    let key_code = event.code();
                    if key_code == ENTER_KEY {
                        state.create_new_todo();
                    }
                }),
                on_input_string =
                    ccomp.callback_arg(|state, value| state.set_new_todo_title(value)),
            ),
        )
    }
}

#[new_view]
impl Footer {
    fn create(ccomp: &Comp<App>) {}
    fn update(ustate: &App) {
        let is_empty_list = ustate.data.items.is_empty();
        let item_left = ustate
            .data
            .items
            .iter()
            .filter(|item| !item.completed)
            .count();
        let text = if item_left == 1 {
            " item left"
        } else {
            " items left"
        };
    }
    fn view() {
        footer(
            class = "footer",
            class_if = (is_empty_list, "hidden"),
            span(class = "todo-count", strong(text(item_left), text(text))),
            ul(
                class = "filters",
                li(a(
                    class_if = (ustate.filter == Filter::All, "selected"),
                    href_with_routing = &Filter::All,
                    text("All"),
                )),
                li(a(
                    class_if = (ustate.filter == Filter::Active, "selected"),
                    href_with_routing = &Filter::Active,
                    text("Active"),
                )),
                li(a(
                    class_if = (ustate.filter == Filter::Completed, "selected"),
                    href_with_routing = &Filter::Completed,
                    text("Completed"),
                )),
            ),
            button(
                class = "clear-completed",
                class_if = (
                    ustate.data.items.iter().any(|item| item.completed).not(),
                    "hidden",
                ),
                on_click = ccomp.callback_arg(|state, _| state.clear_completed()),
                text("Clear completed"),
            ),
        )
    }
}

#[keyed_list_item_for]
impl TodoItem {
    fn get_key(&self) -> &u32 {
        &self.id
    }

    fn create(cdata: &Self, cc: &Context<App>) {
        let id = cdata.id;
    }
    fn update(udata: &Self, uc: &Context<App>) {
        let state = uc.state;
        let is_editing_me = state.editing_id == Some(udata.id);
    }
    fn view() {
        li(
            class_if = (udata.completed, "completed"),
            class_if = (is_editing_me, "editing"),
            div(
                class = "view",
                input(
                    r#type = "checkbox",
                    class = "toggle",
                    checked = udata.completed,
                    on_change = cc.comp.callback_arg(move |state, _| state.toggle(id)),
                ),
                label(
                    on_dblclick = cc
                        .comp
                        .callback_arg(move |state, _| state.start_editing(id)),
                    text(&udata.title),
                ),
                button(
                    class = "destroy",
                    on_click = cc.comp.callback_arg(move |state, _| state.remove(id)),
                ),
            ),
            match is_editing_me {
                false => {}
                true => input(
                    class = "edit",
                    autofocus = true,
                    value = &udata.title,
                    on_blur = uc.comp.callback_arg(move |state, event: FocusEvent| {
                        state.end_editing(event.current_target())
                    }),
                    on_keydown = uc.comp.callback_arg(|state, event: KeyboardEvent| {
                        let key_code = event.code();
                        if key_code == ENTER_KEY {
                            state.end_editing(event.current_target());
                        } else if key_code == ESCAPE_KEY {
                            state.cancel_editing();
                        }
                    }),
                ),
            },
        )
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    spair::start_app_with_routing(
        |_| App {
            filter: Filter::All,
            editing_id: None,
            new_todo_title: String::new(),
            data: read_data_from_storage(),
        },
        App::set_filter,
    );
}
