mod child;

use child::{ChildProps, ChildState};
use spair::prelude::*;

pub struct State {
    value: i32,
    value_sent_from_child: Option<i32>,
    child_comp: spair::ChildComp<ChildState>,
}

impl State {
    fn increment(&mut self) {
        self.value += 1;
    }

    fn decrement(&mut self) {
        self.value -= 1;
    }

    pub fn child_value(&mut self, value: i32) {
        self.value_sent_from_child = Some(value);
    }

    fn send_value_to_child(&mut self) {
        let value = self.value;
        self.child_comp
            .comp()
            .callback_once_arg_mut(ChildState::set_value)
            .call(value);
    }
}

impl spair::Component for State {
    type Routes = ();
    fn render(&self, element: spair::Element<Self>) {
        let comp = element.comp();
        element
            .static_nodes()
            .p(|p| {
                p.static_nodes().static_render("root component");
            })
            .update_nodes()
            .p(|p| {
                p.static_render("The value that received from child-components: ")
                    .match_if(|mi| match self.value_sent_from_child {
                        Some(value) => spair::set_arm!(mi).update_render(value).done(),
                        None => spair::set_arm!(mi).update_render("[Not read yet]").done(),
                    });
            })
            .static_render(Button("-", comp.handler_mut(State::decrement)))
            .update_render(self.value)
            .static_render(Button("+", comp.handler_mut(State::increment)))
            .static_render(Button(
                "Send value to the child-component-ref",
                comp.handler_mut(State::send_value_to_child),
            ))
            .horizontal_line()
            .component_ref(&self.child_comp)
            .horizontal_line()
            .component_owned(|_parent_state, parent_comp| {
                let props = ChildProps {
                    title: "child component owned",
                    callback_arg: parent_comp.callback_arg_mut(State::child_value),
                };
                ChildState::with_props(props).with_updater(
                    |parent_state: &Self| parent_state.value,
                    ChildState::set_value,
                )
            })
            .horizontal_line()
            .static_render(
                "Only split your app into child components if you really need to
                do that. You can use `spair::Render` to split your code into
                smaller pieces.",
            )
            .line_break()
            .line_break()
            .static_render(
                "You can manage a child component in your parent-component's
                state by yourself or let `spair` manages it. If you mangage it,
                `spair` expect a `ref` of it, so the method named `.component_ref`.
                Otherwise, `spair` will own and manage the component, so the
                method named `.component_owned`.",
            )
            .line_break()
            .line_break()
            .static_render(
                "With, `.component_ref`, you have to mangage the component by yourself,
                but it's easier for you to decide when to update the child component.
                On the contrary, it's harder for you to stop propagating change to
                child components when `spair` owned them.",
            )
            .line_break()
            .line_break()
            .static_render(
                "If you change the value in the main component. The child-component-owned
                will be update immediately. But with the child-component-ref, it only
                update if you request it.",
            );
    }
}

struct Button<H>(&'static str, H);
impl<C: spair::Component, H: spair::Click> spair::StaticRender<C> for Button<H> {
    fn render(self, nodes: spair::StaticNodes<C>) {
        nodes.button(|b| {
            b.static_attributes()
                .on_click(self.1)
                .static_nodes()
                .static_render(self.0);
        });
    }
}

impl spair::Application for State {
    fn init(comp: &spair::Comp<Self>) -> Self {
        Self {
            value: 42,
            value_sent_from_child: None,
            child_comp: ChildState::with_props(ChildProps {
                title: "child component ref",
                callback_arg: comp.callback_arg_mut(State::child_value),
            }),
        }
    }
}
#[wasm_bindgen(start)]
pub fn start_counter() {
    wasm_logger::init(wasm_logger::Config::default());
    State::mount_to("root");
}
