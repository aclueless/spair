use spair::prelude::*;

struct State {
    value: i32,
}

impl State {
    fn increment(&mut self) {
        self.value += 1;
    }

    fn decrement(&mut self) {
        self.value -= 1;
    }
}

impl spair::Component for State {
    type Routes = ();
    fn render(&self, element: spair::Element<Self>) {
        let comp = element.comp();
        element
            .static_nodes()
            .p(|p| {
                p.static_nodes()
                    .static_text("The initial value is ")
                    .static_text(self.value);
            })
            .update_nodes()
            .rfn(|nodes| render_button("-", comp.handler_mut(State::decrement), nodes))
            .update_text(self.value)
            .rfn(|nodes| render_button("+", comp.handler_mut(State::increment), nodes));
    }
}

fn render_button<H: spair::Click>(label: &str, handler: H, nodes: spair::Nodes<State>) {
    nodes.static_nodes().button(|b| {
        b.static_attributes()
            .on_click(handler)
            .static_nodes()
            .static_text(label);
    });
}

impl spair::Application for State {
    fn init(_: &spair::Comp<Self>) -> Self {
        Self { value: 42 }
    }
}

#[wasm_bindgen(start)]
pub fn start_counter() {
    State::mount_to_element_id("root");
}
