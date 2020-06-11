// https://github.com/tastejs/todomvc-app-template/

mod utils;

use serde::{Deserialize, Serialize};
use spair::prelude::*;

#[derive(PartialEq, Clone, Copy)]
enum Filter {
    All,
    Active,
    Completed,
}

impl Default for Filter {
    fn default() -> Self {
        Self::All
    }
}

impl Filter {
    fn as_str(&self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Active => "Active",
            Self::Completed => "Completed",
        }
    }
}

impl spair::Routes<AppState> for Filter {
    fn url(&self) -> String {
        match self {
            Self::All => "#all".to_string(),
            Self::Active => "#active".to_string(),
            Self::Completed => "#completed".to_string(),
        }
    }
    fn routing(location: spair::Location, comp: &spair::Comp<AppState>) {
        let filter = match location.hash().unwrap_or_else(|_| String::new()).as_str() {
            "#completed" => Self::Completed,
            "#active" => Self::Active,
            _ => Self::All,
        };
        comp.callback_arg(AppState::set_filter)(filter);
    }
}

#[derive(Serialize, Deserialize)]
struct TodoItem {
    id: u32,
    title: String,
    completed: bool,
}

impl TodoItem {
    fn visible(&self, filter: &Filter) -> bool {
        match (filter, self.completed) {
            (Filter::All, _) => true,
            (Filter::Active, false) => true,
            (Filter::Completed, true) => true,
            _ => false,
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
struct TodoData {
    next_id: u32,
    items: Vec<TodoItem>,
}

struct AppState {
    data: TodoData,

    filter: Filter,
    editing_id: Option<u32>,
    new_todo_title: String,
}

impl AppState {
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
        utils::write_data_to_storage(&self.data);
    }

    fn toggle_all(&mut self, checked: bool) {
        self.data
            .items
            .iter_mut()
            .for_each(|item| item.completed = checked);
        utils::write_data_to_storage(&self.data);
    }

    fn toggle(&mut self, id: u32) {
        if let Some(item) = self.data.items.iter_mut().find(|item| item.id == id) {
            item.completed = !item.completed;
            utils::write_data_to_storage(&self.data);
        }
    }

    fn clear_completed(&mut self) {
        self.data.items.retain(|item| !item.completed);
        utils::write_data_to_storage(&self.data);
    }

    fn remove(&mut self, id: u32) {
        self.data.items.retain(|item| item.id != id);
        utils::write_data_to_storage(&self.data);
    }

    fn start_editing(&mut self, id: u32) {
        self.editing_id = Some(id);
    }

    fn end_editing(&mut self, title: Option<String>) {
        let id = match self.editing_id {
            Some(id) => id,
            None => return,
        };
        match title {
            Some(title) => {
                self.data
                    .items
                    .iter_mut()
                    .find(|item| item.id == id)
                    .expect_throw("Why editing item with an invalid id?")
                    .title = title;
                utils::write_data_to_storage(&self.data);
            }
            None => self.remove(id),
        }
        self.editing_id = None;
    }

    fn cancel_editing(&mut self) {
        self.editing_id = None;
    }
}

impl spair::Component for AppState {
    type Routes = Filter;
    fn with_comp(_: spair::Comp<Self>) -> Option<Self> {
        Some(Self {
            data: utils::read_data_from_storage(),
            filter: Filter::default(),
            editing_id: None,
            new_todo_title: String::new(),
        })
    }
    fn render(&self, c: spair::Context<Self>) {
        let (_, element) = c.into_parts();
        element
            .nodes()
            .section(|s| {
                s.static_attributes()
                    .class("todoapp")
                    .nodes()
                    .render(Header(self))
                    .render(Main(self))
                    .render(Footer(self));
            })
            .render(Info);
    }
}

struct Header<'s>(&'s AppState);
impl<'s> spair::Render<AppState> for Header<'s> {
    fn render(self, nodes: spair::Nodes<AppState>) -> spair::Nodes<AppState> {
        let comp = nodes.comp();
        nodes.header(|h| {
            h.static_attributes()
                .class("header")
                .static_nodes()
                .h1(|h| h.nodes().render("Spair Todos").done())
                .nodes()
                .input(|i| {
                    i.static_attributes()
                        .class("new-todo")
                        .focus(true)
                        .placeholder("What needs to be done?")
                        .on_input(comp.handler_arg(|state, arg: web_sys::InputEvent| {
                            let input =
                                spair::into_input(arg.target().expect_throw("No event target"));
                            state.set_new_todo_title(input.value());
                        }))
                        .on_key_press(comp.handler_arg(|state, arg: web_sys::KeyboardEvent| {
                            // `.key_code()` is deprecated, so we use code instead
                            // https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/keyCode
                            match arg.code().as_str() {
                                "Enter" => state.create_new_todo(),
                                _ => {}
                            }
                        }))
                        .attributes()
                        .value(&self.0.new_todo_title);
                });
        })
    }
}

struct Main<'s>(&'s AppState);
impl<'s> spair::Render<AppState> for Main<'s> {
    fn render(self, nodes: spair::Nodes<AppState>) -> spair::Nodes<AppState> {
        let comp = nodes.comp();
        let todo_count = self.0.data.items.len();
        let all_completed = self.0.data.items.iter().all(|item| item.completed);
        nodes.section(|s| {
            s.static_attributes()
                .class("main")
                .attributes()
                .class_if("hidden", todo_count == 0)
                .nodes()
                .input(move |i| {
                    i.static_attributes()
                        .id("toggle-all")
                        .class("toggle-all")
                        .r#type(spair::InputType::CheckBox)
                        .attributes()
                        .checked(all_completed)
                        .on_change(comp.handler(move |state| state.toggle_all(!all_completed)));
                })
                .static_nodes()
                .label(|l| {
                    l.static_attributes()
                        .r#for("toggle-all")
                        .static_nodes()
                        .render("Mark all as complete");
                })
                .nodes()
                .ul(|u| {
                    u.static_attributes().class("todo-list").list(
                        Some(self.0),
                        self.0
                            .data
                            .items
                            .iter()
                            .filter(|item| item.visible(&self.0.filter)),
                        spair::ListElementCreation::Clone,
                    )
                });
        })
    }
}

struct Footer<'s>(&'s AppState);
impl<'s> spair::Render<AppState> for Footer<'s> {
    fn render(self, nodes: spair::Nodes<AppState>) -> spair::Nodes<AppState> {
        let comp = nodes.comp();
        let list_empty = self.0.data.items.len() == 0;
        let item_left = self
            .0
            .data
            .items
            .iter()
            .filter(|item| !item.completed)
            .count();
        let some_completed = self.0.data.items.iter().any(|item| item.completed);
        nodes.footer(|f| {
            f.static_attributes()
                .class("footer")
                .attributes()
                .class_if("hidden", list_empty)
                .nodes()
                .span(|s| {
                    s.static_attributes()
                        .class("todo-count")
                        .nodes()
                        .strong(|s| s.nodes().render(item_left).done())
                        .render(if item_left == 1 {
                            " item left"
                        } else {
                            " items left"
                        });
                })
                .ul(|u| {
                    u.static_attributes()
                        .class("filters")
                        .nodes()
                        .render(FilterView {
                            current_filter: self.0.filter,
                            view: Filter::All,
                        })
                        .render(FilterView {
                            current_filter: self.0.filter,
                            view: Filter::Active,
                        })
                        .render(FilterView {
                            current_filter: self.0.filter,
                            view: Filter::Completed,
                        });
                })
                .button(|b| {
                    b.static_attributes()
                        .class("clear-completed")
                        .on_click(comp.handler(AppState::clear_completed))
                        .attributes()
                        .class_if("hidden", !some_completed)
                        .static_nodes()
                        .r#static("Clear completed");
                });
        })
    }
}

struct FilterView {
    current_filter: Filter,
    view: Filter,
}

impl spair::Render<AppState> for FilterView {
    fn render(self, nodes: spair::Nodes<AppState>) -> spair::Nodes<AppState> {
        nodes.li(|l| {
            l.nodes().a(|a| {
                a.static_attributes()
                    .href(&self.view)
                    .attributes()
                    .class_if("selected", self.current_filter == self.view)
                    .static_nodes()
                    .r#static(self.view.as_str());
            });
        })
    }
}

struct Info;
impl spair::Render<AppState> for Info {
    fn render(self, nodes: spair::Nodes<AppState>) -> spair::Nodes<AppState> {
        nodes.footer(|f| {
            f.static_attributes()
                .class("info")
                .static_nodes()
                .p(|p| p.nodes().render("Double-click to edit a todo").done())
                .p(|p| p.nodes().render("Created by 'aclueless'").done())
                .p(|p| {
                    p.nodes().render("Part of ").a(|a| {
                        a.static_attributes()
                            .href_str("http://todomvc.com")
                            .nodes()
                            .render("TodoMVC");
                    });
                });
        })
    }
}

impl spair::ListItem<AppState> for TodoItem {
    const ROOT_ELEMENT_TAG: &'static str = "li";
    fn render(&self, state: Option<&AppState>, li: spair::Element<AppState>) {
        let comp = li.comp();
        let comp = &comp;
        let id = self.id;
        let is_editing_me = state.and_then(|s| s.editing_id) == Some(self.id);
        li.attributes()
            .class_if("completed", self.completed)
            .class_if("editing", is_editing_me)
            .nodes()
            .div(move |d| {
                d.static_attributes()
                    .class("view")
                    .nodes()
                    .input(|i| {
                        i.static_attributes()
                            .class("toggle")
                            .r#type(spair::InputType::CheckBox)
                            .attributes()
                            .on_change(comp.handler(move |state| state.toggle(id)))
                            .checked(self.completed);
                    })
                    .label(|l| {
                        l.attributes()
                            .on_double_click(comp.handler(move |state| state.start_editing(id)))
                            .nodes()
                            .render(&self.title);
                    })
                    .button(|b| {
                        b.static_attributes()
                            .class("destroy")
                            .attributes()
                            .on_click(comp.handler(move |state| state.remove(id)));
                    });
            })
            .match_if(|arm| match is_editing_me {
                true => arm
                    .render_on_arm_index(0)
                    .render(EditingInput(&self.title))
                    .done(),
                false => arm.render_on_arm_index(1).done(),
            });
    }
}

struct EditingInput<'a>(&'a String);
impl<'a> spair::Render<AppState> for EditingInput<'a> {
    fn render(self, nodes: spair::Nodes<AppState>) -> spair::Nodes<AppState> {
        let comp = nodes.comp();
        nodes.input(|i| {
            i.static_attributes()
                .class("edit")
                .on_blur(comp.handler_arg(|state, arg: web_sys::FocusEvent| {
                    state.end_editing(get_value(spair::into_input(
                        arg.target().expect_throw("No event target"),
                    )))
                }))
                .on_key_down(comp.handler_arg(|state, arg: web_sys::KeyboardEvent| {
                    // `.key_code()` is deprecated, so we use code instead
                    // https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/keyCode
                    match arg.code().as_str() {
                        "Escape" => state.cancel_editing(),
                        "Enter" => state.end_editing(get_value(spair::into_input(
                            arg.target().expect_throw("No event target"),
                        ))),
                        _ => {}
                    }
                }))
                .attributes()
                .focus(true)
                .value(self.0);
        })
    }
}

fn get_value(i: web_sys::HtmlInputElement) -> Option<String> {
    let s = i.value();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

#[wasm_bindgen(start)]
pub fn start_todo_mvc() {
    // wasm_logger::init(wasm_logger::Config::default());
    AppState::mount_to_body();
}
