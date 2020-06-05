mod utils;

use serde::{Deserialize, Serialize};
use spair::prelude::*;

#[derive(PartialEq)]
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

impl spair::Routes<State> for Filter {
    fn url(&self) -> String {
        match self {
            Self::All => "#all".to_string(),
            Self::Active => "#active".to_string(),
            Self::Completed => "#completed".to_string(),
        }
    }
    fn routing(location: spair::Location, comp: &spair::Comp<State>) {
        let filter = match location.hash().unwrap_or_else(|_| String::new()).as_str() {
            "#completed" => Self::Completed,
            "#active" => Self::Active,
            _ => Self::All,
        };
        comp.update_arg(filter, &std::rc::Rc::new(State::set_filter));
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
struct State {
    next_id: u32,
    items: Vec<TodoItem>,

    #[serde(skip)]
    editing: Option<u32>,

    #[serde(skip)]
    filter: Filter,
}

impl State {
    fn from_store() -> Self {
        utils::read_data_from_storage()
    }

    fn save_to_local_storage(&self) {
        utils::write_data_to_storage(self);
    }

    fn set_filter(&mut self, filter: Filter) {
        self.filter = filter;
    }

    fn add_new_todo(&mut self, title: String) {
        self.items.push(TodoItem {
            id: self.next_id,
            title,
            completed: false,
        });
        self.next_id += 1;
        self.save_to_local_storage();
    }

    fn toggle_all(&mut self, checked: bool) {
        self.items
            .iter_mut()
            .for_each(|item| item.completed = checked);
        self.save_to_local_storage();
    }

    fn toggle(&mut self, id: u32) {
        if let Some(item) = self.items.iter_mut().find(|item| item.id == id) {
            item.completed = !item.completed;
            self.save_to_local_storage();
        }
    }

    fn clear_completed(&mut self) {
        self.items.retain(|item| !item.completed);
        self.save_to_local_storage();
    }

    fn remove(&mut self, id: u32) {
        self.items.retain(|item| item.id != id);
        self.save_to_local_storage();
    }

    fn start_editing(&mut self, id: u32) {
        self.editing = Some(id);
    }

    fn end_editing(&mut self, title: Option<String>) {
        let id = match self.editing {
            Some(id) => id,
            None => return,
        };
        match title {
            Some(title) => {
                self.items
                    .iter_mut()
                    .find(|item| item.id == id)
                    .expect_throw("Why editing item with an invalid id?")
                    .title = title;
                self.save_to_local_storage();
            }
            None => self.remove(id),
        }
        self.editing = None;
    }
}

impl spair::Component for State {
    type Routes = Filter;
    fn with_comp(_: spair::Comp<Self>) -> Option<Self> {
        Some(Self::from_store())
    }
    fn render(&self, c: spair::Context<Self>) {
        let (_, element) = c.into_parts();
        element
            .nodes()
            .section(|s| {
                s.static_attributes()
                    .class("todoapp")
                    .nodes()
                    .render(Header)
                    .render(Main(self))
                    .render(Footer(self));
            })
            .render(Info);
    }
}

struct Header;
impl spair::Render<State> for Header {
    fn render(self, nodes: spair::Nodes<State>) -> spair::Nodes<State> {
        let comp = nodes.comp();
        nodes.header(|h| {
            h.static_attributes()
                .class("header")
                .static_nodes()
                .h1(|h| h.nodes().render("Spair Todos").done())
                .input(|i| {
                    i.static_attributes()
                        .class("new-todo")
                        .focus(true)
                        .placeholder("What needs to be done?")
                        .on_change(comp.handler_arg(|state, arg: web_sys::Event| {
                            let input =
                                spair::into_input(arg.target().expect_throw("No event target"));
                            state.add_new_todo(input.value());
                        }));
                });
        })
    }
}

struct Main<'s>(&'s State);
impl<'s> spair::Render<State> for Main<'s> {
    fn render(self, nodes: spair::Nodes<State>) -> spair::Nodes<State> {
        let comp = nodes.comp();
        let todo_count = self.0.items.len();
        let all_completed = self.0.items.iter().all(|item| item.completed);
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
                .label(|l| {
                    l.static_attributes()
                        .r#for("toggle-all")
                        .static_nodes()
                        .render("Mark all as complete");
                })
                .ul(|u| {
                    u.static_attributes().class("todo-list").keyed_list(
                        Some(self.0),
                        self.0
                            .items
                            .iter()
                            .filter(|item| item.visible(&self.0.filter)),
                        spair::ListElementCreation::Clone,
                    )
                });
        })
    }
}

struct Footer<'s>(&'s State);
impl<'s> spair::Render<State> for Footer<'s> {
    fn render(self, nodes: spair::Nodes<State>) -> spair::Nodes<State> {
        let comp = nodes.comp();
        let list_empty = self.0.items.len() == 0;
        let item_left = self.0.items.iter().filter(|item| !item.completed).count();
        let some_completed = self.0.items.iter().any(|item| item.completed);
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
                        .strong(|s| {
                            s.nodes().render(item_left).render(if item_left == 1 {
                                " item left"
                            } else {
                                " items left"
                            });
                        });
                })
                .ul(|u| {
                    u.static_attributes()
                        .class("filters")
                        .nodes()
                        .li(|l| {
                            l.nodes().a(|a| {
                                a.static_attributes()
                                    .href(Filter::All)
                                    .attributes()
                                    .class_if("selected", self.0.filter == Filter::All)
                                    .static_nodes()
                                    .r#static("All");
                            });
                        })
                        .li(|l| {
                            l.nodes().a(|a| {
                                a.static_attributes()
                                    .href(Filter::Active)
                                    .attributes()
                                    .class_if("selected", self.0.filter == Filter::Active)
                                    .static_nodes()
                                    .r#static("Active");
                            });
                        })
                        .li(|l| {
                            l.nodes().a(|a| {
                                a.static_attributes()
                                    .href(Filter::Completed)
                                    .attributes()
                                    .class_if("selected", self.0.filter == Filter::Completed)
                                    .static_nodes()
                                    .r#static("Completed");
                            });
                        });
                })
                .button(|b| {
                    b.static_attributes()
                        .class("clear-completed")
                        .on_click(comp.handler(State::clear_completed))
                        .attributes()
                        .class_if("hidden", !some_completed)
                        .static_nodes()
                        .r#static("Clear completed");
                });
        })
    }
}

struct Info;
impl spair::Render<State> for Info {
    fn render(self, nodes: spair::Nodes<State>) -> spair::Nodes<State> {
        nodes.footer(|f| {
            f.static_attributes()
                .class("info")
                .static_nodes()
                .p(|p| p.nodes().render("Double-click to edit a todo").done())
                .p(|p| p.nodes().render("Created by 'aclueless'").done())
                .p(|p| p.nodes().render("Part of Spair").done())
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

impl spair::KeyedListItem<'_, State> for TodoItem {
    type Key = u64;
    fn key(&self) -> Self::Key {
        self.id as u64
    }
}

impl spair::ListItem<State> for TodoItem {
    const ROOT_ELEMENT_TAG: &'static str = "li";
    fn render(&self, state: Option<&State>, li: spair::Element<State>) {
        let comp = li.comp();
        let comp = &comp;
        let id = self.id;
        let is_is_me = state.and_then(|s| s.editing) == Some(self.id);
        li.attributes()
            .class_if("completed", self.completed)
            .class_if("editing", is_is_me)
            .nodes()
            .div(move |d| {
                d.static_attributes()
                    .class("filter")
                    .nodes()
                    .input(|i| {
                        i.static_attributes()
                            .class("toggle")
                            .r#type(spair::InputType::CheckBox)
                            .on_change(comp.handler(move |state| state.toggle(id)))
                            .attributes()
                            .checked(self.completed);
                    })
                    .label(|l| {
                        l.static_attributes()
                            .on_double_click(comp.handler(move |state| state.start_editing(id)))
                            .nodes()
                            .render(&self.title);
                    })
                    .button(|b| {
                        b.static_attributes()
                            .class("destroy")
                            .on_click(comp.handler(move |state| state.remove(id)));
                    });
            })
            .input(|i| {
                i.static_attributes()
                    .class("edit")
                    .on_blur(comp.handler_arg(|state, arg: web_sys::FocusEvent| {
                        state.end_editing(get_value(spair::into_input(
                            arg.target().expect_throw("No event target"),
                        )))
                    }))
                    .on_key_press(comp.handler_arg(|state, arg: web_sys::KeyboardEvent| {
                        match arg.key().as_str() {
                            "Escape" => state.end_editing(None),
                            "Enter" => state.end_editing(get_value(spair::into_input(
                                arg.target().expect_throw("No event target"),
                            ))),
                            _ => {}
                        }
                    }))
                    .attributes()
                    .focus(is_is_me)
                    .value(&self.title);
            });
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
    State::mount_to("root");
}
