mod child;

use child::ChildState;
use spair::prelude::*;

pub struct State {
    value: i32,
    value_read_from_child: Option<i32>,
}

impl State {
    fn increment(&mut self) {
        self.value += 1;
    }

    fn decrement(&mut self) {
        self.value -= 1;
    }

    pub fn child_value_is_divisible_by_five(&mut self, child_comps: &mut ChildComp) {
        self.value_read_from_child = Some(child_comps.0.comp_instance().state().value());
    }

    fn send_value_to_child(&mut self, child_comps: &mut ChildComp) {
        let value = self.value;
        spair::update_component(
            child_comps
                .0
                .comp()
                .callback(move |state| state.set_value(value)),
        );
    }
}

pub struct ChildComp(spair::ChildComp<ChildState>);

impl spair::Components<State> for ChildComp {
    fn new(_: &State, parent_comp: spair::Comp<State>) -> Self {
        Self(ChildState::new(parent_comp).into())
    }
}

impl spair::Component for State {
    type Routes = ();
    type Components = ChildComp;
    fn render(&self, c: spair::Context<Self>) {
        let (comp, element, child) = c.into_parts();
        element
            .nodes()
            .div(|d| d.component(&child.0))
            .static_nodes()
            .p(|p| {
                p.static_nodes().render(
                    "This line and everything below is in the main-component",
                );
            })
            .nodes()
            .p(|p| {
                p.nodes()
                    .render("The value that read from child component: ")
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
                comp.handler_child_comps(State::send_value_to_child),
            ));
    }
}

struct Button<H>(&'static str, H);
impl<C: spair::Component, H: spair::Click> spair::StaticRender<C> for Button<H> {
    fn render(self, nodes: spair::StaticNodes<C>) -> spair::StaticNodes<C> {
        nodes.button(|b| {
            b.static_attributes()
                .on_click(self.1)
                .static_nodes()
                .r#static(self.0);
        })
    }
}

#[wasm_bindgen(start)]
pub fn start_counter() {
    wasm_logger::init(wasm_logger::Config::default());
    let state = State {
        value: 42,
        value_read_from_child: None,
    };
    spair::start(state, "root");
}
