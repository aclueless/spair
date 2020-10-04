mod child;

use child::ChildState;
use spair::prelude::*;

pub struct State {
    value: i32,
    value_read_from_child: Option<i32>,
    child_comp: spair::ChildComp<ChildState>,
}

impl State {
    fn increment(&mut self) {
        self.value += 1;
    }

    fn decrement(&mut self) {
        self.value -= 1;
    }

    pub fn child_value_is_divisible_by_five(&mut self) {
        self.value_read_from_child = Some(self.child_comp.comp_instance().state().value());
    }

    fn send_value_to_child(&mut self) {
        let value = self.value;
        spair::update_component(
            self.child_comp
                .comp()
                .callback(move |state| state.set_value(value)),
        );
    }
}

impl spair::Component for State {
    type Routes = ();
    fn with_comp(comp: spair::Comp<Self>) -> Option<Self> {
        Some(Self {
            value: 42,
            value_read_from_child: None,
            child_comp: ChildState::new(comp).into(),
        })
    }

    fn render(&self, element: spair::Element<Self>) {
        let comp = element.comp();
        element
            .nodes()
            .div(|d| d.component(&self.child_comp))
            .static_nodes()
            .p(|p| {
                p.static_nodes()
                    .r#static("This line and everything below is in the main-component");
            })
            .nodes()
            .p(|p| {
                p.nodes()
                    .r#static("The value that read from child component: ")
                    .match_if(|arm| match self.value_read_from_child {
                        Some(value) => arm.render_on_arm_index(0).render(value).done(),
                        None => arm.render_on_arm_index(1).render("[Not read yet]").done(),
                    });
            })
            .r#static(Button("-", comp.handler(State::decrement)))
            .render(self.value)
            .r#static(Button("+", comp.handler(State::increment)))
            .r#static(Button(
                "Send value to child-component",
                comp.handler(State::send_value_to_child),
            ));
    }
}

struct Button<H>(&'static str, H);
impl<C: spair::Component, H: spair::Click> spair::StaticRender<C> for Button<H> {
    fn render(self, nodes: spair::StaticNodes<C>) {
        nodes.button(|b| {
            b.static_attributes()
                .on_click(self.1)
                .static_nodes()
                .r#static(self.0);
        });
    }
}

#[wasm_bindgen(start)]
pub fn start_counter() {
    State::mount_to("root");
}
