use spair::prelude::*;

struct State {
    rate: i32,
    value: spair::QrVal<i32>,
}

impl State {
    fn increment(&mut self) {
        self.value.set_with(|v| v + 1);
    }

    fn decrement(&mut self) {
        self.value.set_with(|v| v - 1);
    }
}

impl spair::Component for State {
    type Routes = ();
    fn default_should_render() -> spair::ShouldRender {
        spair::ShouldRender::No
    }

    fn render(&self, element: spair::Element<Self>) {
        let comp = element.comp();
        element
            .static_nodes()
            .p(|p| {
                p.static_nodes()
                    .static_text("The initial value is ")
                    .static_text(self.value.get())
                    .line_break()
                    .static_text("The rate is ")
                    .static_text(self.rate)
                    ;
            })
            .static_text("Value: ")
            .update_text(&self.value)
            .line_break()
            .static_text("Value / self.rate = ")
            // Unfortunately, Rust fails to inference types for this closure
            .update_text(self.value.map_with_state(|state: &Self, value: &i32| value / state.rate))
            .line_break()
            .static_text("Value * self.rate = ")
            .update_text(self.value.map_with_state(|state: &Self, value: &i32| value * state.rate))
            .line_break()
            .update_nodes()
            .rfn(|nodes| render_button("-", comp.handler_mut(State::decrement), nodes))
            .rfn(|nodes| render_button("+", comp.handler_mut(State::increment), nodes))
            .line_break()
            .static_text("This value will be never updated if the update method return `spair::ShouldRender::No`: ")
            .update_text(self.value.get())
            ;
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
        Self {
            rate: 3,
            value: 42.into(),
        }
    }
}

#[wasm_bindgen(start)]
pub fn start_counter() {
    wasm_logger::init(wasm_logger::Config::default());
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    State::mount_to_element_id("root");
}
