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

struct Router {
    comp: spair::Comp<App>,
}

impl spair::Router for Router {
    fn routing(&self, location: web_sys::Location) {
        let filter = match location.hash().unwrap_or_else(|_| String::new()).as_str() {
            "#completed" => Filter::Completed,
            "#active" => Filter::Active,
            _ => Filter::All,
        };
        self.comp.callback_arg_mut(App::set_filter)(filter);
    }
}

impl spair::Routes for Filter {
    type Router = Router;
    fn url(&self) -> String {
        match self {
            Self::All => "#all".to_string(),
            Self::Active => "#active".to_string(),
            Self::Completed => "#completed".to_string(),
        }
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
        matches!(
            (filter, self.completed),
            (Filter::All, _) | (Filter::Active, false) | (Filter::Completed, true)
        )
    }
}

#[derive(Default, Serialize, Deserialize)]
struct TodoData {
    next_id: u32,
    items: Vec<TodoItem>,
}

struct App {
    data: TodoData,

    filter: Filter,
    editing_id: Option<u32>,
    new_todo_title: String,
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

impl spair::Component for App {
    type Routes = Filter;

    fn render(&self, element: spair::Element<Self>) {
        element
            .section(|s| {
                s.static_attributes()
                    .class("todoapp")
                    .render(Header)
                    .render(Main)
                    .render(Footer);
            })
            .render(Info);
    }
}

struct Header;
impl spair::Render<App> for Header {
    fn render(self, nodes: spair::Nodes<App>) {
        let comp = nodes.comp();
        let state = nodes.state();
        nodes.header(|h| {
            h.static_attributes()
                .class("header")
                .static_nodes()
                .h1(|h| h.render("Spair Todos").done())
                .nodes()
                .input(|i| {
                    i.value(&state.new_todo_title)
                        .static_attributes()
                        .class("new-todo")
                        .focus(true)
                        .placeholder("What needs to be done?")
                        .on_input(comp.handler_arg_mut(|state, arg: spair::InputEvent| {
                            if let Some(input) = arg.target_as_input_element() {
                                state.set_new_todo_title(input.value());
                            }
                        }))
                        .on_key_press(comp.handler_arg_mut(|state, arg: spair::KeyboardEvent| {
                            // `.key_code()` is deprecated, so we use code instead
                            // https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/keyCode
                            if arg.raw().code().as_str() == "Enter" {
                                state.create_new_todo();
                            }
                        }));
                });
        });
    }
}

struct Main;
impl spair::Render<App> for Main {
    fn render(self, nodes: spair::Nodes<App>) {
        let comp = nodes.comp();
        let state = nodes.state();
        let todo_count = state.data.items.len();
        let all_completed = state.data.items.iter().all(|item| item.completed);
        nodes.section(|s| {
            s.class_if("hidden", todo_count == 0)
                .static_attributes()
                .class("main")
                .input(move |i| {
                    i.checked(all_completed)
                        .on_change(comp.handler_mut(move |state| state.toggle_all(!all_completed)))
                        .static_attributes()
                        .id("toggle-all")
                        .class("toggle-all")
                        .r#type(spair::InputType::CheckBox);
                })
                .static_nodes()
                .label(|l| {
                    l.static_attributes()
                        .r#for("toggle-all")
                        .static_nodes()
                        .r#static("Mark all as complete");
                })
                .nodes()
                .ul(|u| {
                    u.static_attributes().class("todo-list").list(
                        state
                            .data
                            .items
                            .iter()
                            .filter(|item| item.visible(&state.filter)),
                        spair::ListElementCreation::Clone,
                    );
                });
        });
    }
}

struct Footer;
impl spair::Render<App> for Footer {
    fn render(self, nodes: spair::Nodes<App>) {
        let comp = nodes.comp();
        let state = nodes.state();
        let list_empty = state.data.items.is_empty();
        let item_left = state
            .data
            .items
            .iter()
            .filter(|item| !item.completed)
            .count();
        let some_completed = state.data.items.iter().any(|item| item.completed);
        nodes.footer(|f| {
            f.class_if("hidden", list_empty)
                .static_attributes()
                .class("footer")
                .span(|s| {
                    s.static_attributes()
                        .class("todo-count")
                        .strong(|s| s.render(item_left).done())
                        .render(if item_left == 1 {
                            " item left"
                        } else {
                            " items left"
                        });
                })
                .ul(|u| {
                    u.static_attributes()
                        .class("filters")
                        .render(FilterView {
                            current_filter: state.filter,
                            view: Filter::All,
                        })
                        .render(FilterView {
                            current_filter: state.filter,
                            view: Filter::Active,
                        })
                        .render(FilterView {
                            current_filter: state.filter,
                            view: Filter::Completed,
                        });
                })
                .button(|b| {
                    b.class_if("hidden", !some_completed)
                        .static_attributes()
                        .class("clear-completed")
                        .on_click(comp.handler_mut(App::clear_completed))
                        .r#static("Clear completed");
                });
        });
    }
}

struct FilterView {
    current_filter: Filter,
    view: Filter,
}

impl spair::Render<App> for FilterView {
    fn render(self, nodes: spair::Nodes<App>) {
        nodes.li(|l| {
            l.a(|a| {
                a.class_if("selected", self.current_filter == self.view)
                    .static_attributes()
                    .href(&self.view)
                    .static_nodes()
                    .r#static(self.view.as_str());
            });
        });
    }
}

struct Info;
impl spair::Render<App> for Info {
    fn render(self, nodes: spair::Nodes<App>) {
        nodes.footer(|f| {
            f.static_attributes()
                .class("info")
                .static_nodes()
                .p(|p| p.r#static("Double-click to edit a todo").done())
                .p(|p| p.r#static("Created by 'aclueless'").done())
                .p(|p| {
                    p.r#static("Part of ").a(|a| {
                        a.static_attributes()
                            .href_str("http://todomvc.com")
                            .r#static("TodoMVC");
                    });
                });
        });
    }
}

impl spair::ListItemRender<App> for &TodoItem {
    const ROOT_ELEMENT_TAG: &'static str = "li";
    fn render(self, li: spair::Element<App>) {
        let comp = li.comp();
        let state = li.state();
        let id = self.id;
        let is_editing_me = state.editing_id == Some(self.id);
        li.class_if("completed", self.completed)
            .class_if("editing", is_editing_me)
            .div(move |d| {
                d.static_attributes()
                    .class("view")
                    .input(|i| {
                        i.on_change(comp.handler_mut(move |state| state.toggle(id)))
                            .checked(self.completed)
                            .static_attributes()
                            .class("toggle")
                            .r#type(spair::InputType::CheckBox);
                    })
                    .label(|l| {
                        l.on_double_click(comp.handler_mut(move |state| state.start_editing(id)))
                            .render(&self.title);
                    })
                    .button(|b| {
                        b.on_click(comp.handler_mut(move |state| state.remove(id)))
                            .static_attributes()
                            .class("destroy");
                    });
            })
            .match_if(|mi| match is_editing_me {
                true => spair::set_arm!(mi).render(EditingInput(&self.title)).done(),
                false => spair::set_arm!(mi).done(),
            });
    }
}

struct EditingInput<'a>(&'a String);
impl<'a> spair::Render<App> for EditingInput<'a> {
    fn render(self, nodes: spair::Nodes<App>) {
        let comp = nodes.comp();
        nodes.input(|i| {
            i.focus(true)
                .value(self.0)
                .static_attributes()
                .class("edit")
                .on_blur(comp.handler_arg_mut(|state, arg: spair::FocusEvent| {
                    state.end_editing(get_value(arg.target_as()))
                }))
                .on_key_down(comp.handler_arg_mut(|state, arg: spair::KeyboardEvent| {
                    // `.key_code()` is deprecated, so we use code instead
                    // https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/keyCode
                    match arg.raw().code().as_str() {
                        "Escape" => state.cancel_editing(),
                        "Enter" => state.end_editing(get_value(arg.target_as())),
                        _ => {}
                    }
                }));
        });
    }
}

fn get_value(i: Option<web_sys::HtmlInputElement>) -> Option<String> {
    i.and_then(|i| {
        let text = i.value();
        let text = text.trim();
        match text.is_empty() {
            true => None,
            false => Some(text.to_string()),
        }
    })
}

impl spair::Application for App {
    fn init(_: &spair::Comp<Self>) -> Self {
        Self {
            data: utils::read_data_from_storage(),
            filter: Filter::default(),
            editing_id: None,
            new_todo_title: String::new(),
        }
    }

    fn init_router(comp: &spair::Comp<Self>) -> Option<Router> {
        Some(Router { comp: comp.clone() })
    }
}

#[wasm_bindgen(start)]
pub fn start_todo_mvc() {
    App::mount_to_body();
}
