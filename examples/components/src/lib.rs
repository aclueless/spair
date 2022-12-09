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
            .call_or_queue(value);
    }
}

impl spair::Component for State {
    type Routes = ();
    fn render(&self, element: spair::Element<Self>) {
        let comp = element.comp();
        element
            .static_nodes()
            .rstatic(
                "Using Spair, you don't usually use components. If you only need
                to split your code to smaller spieces, then use `spair::Render`.
                Only split your app into child components if you really need to
                do that.",
            )
            .horizontal_line()
            .rstatic("root component")
            .update_nodes()
            .div(|d| {
                d.rstatic("The value that received from child-components: ")
                    .match_if(|mi| match self.value_sent_from_child {
                        Some(value) => spair::set_arm!(mi).rupdate(value).done(),
                        None => spair::set_arm!(mi).rupdate("[not yet]").done(),
                    });
            })
            .line_break()
            .rstatic(Button("-", comp.handler_mut(State::decrement)))
            .rupdate(self.value)
            .rstatic(Button("+", comp.handler_mut(State::increment)))
            .rstatic(Button(
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
                    description: "With `.component_owned`, the child component is created
                    and updated in render-phase. Spair have to store the component in it's
                    DOM tree, hence the method name is `.component_owned`. The update
                    method is ran on every execution of render, so it's harder for you
                    to control the update of the child component.",
                };
                ChildState::with_props(props).with_updater(
                    |parent_state: &Self| parent_state.value,
                    ChildState::set_value,
                )
            });
    }
}

struct Button<H>(&'static str, H);
impl<C: spair::Component, H: spair::Click> spair::StaticRender<C> for Button<H> {
    fn render(self, nodes: spair::StaticNodes<C>) {
        nodes.button(|b| {
            b.static_attributes()
                .on_click(self.1)
                .static_nodes()
                .rstatic(self.0);
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
                description: "With `.component_ref`, you have to manage the child
                component (in the parent component's state) by yourself. Spair's
                DOM tree only access to a ref of the child component, hence the
                method name is `component_ref`. It's easier for you to decide when
                to update the child component.",
            }),
        }
    }
}
#[wasm_bindgen(start)]
pub fn start_counter() {
    wasm_logger::init(wasm_logger::Config::default());
    State::mount_to_element_id("root");
}
