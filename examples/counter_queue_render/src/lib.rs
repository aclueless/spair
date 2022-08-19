use spair::prelude::*;

struct State {
    rate: i32,
    value: spair::Value<i32>,
}

impl State {
    fn increment(&mut self) {
        self.value.set_with(|v| *v + 1);
    }

    fn decrement(&mut self) {
        self.value.set_with(|v| *v - 1);
    }
}

impl spair::Component for State {
    type Routes = ();
    fn test_state() -> Option<Self> {
        Some(State {
            rate: 2,
            value: 4.into(),
        })
    }

    fn default_should_render() -> spair::ShouldRender {
        spair::ShouldRender::No
    }

    fn render(&self, element: spair::Element<Self>) {
        let comp = element.comp();
        element
            .static_nodes()
            .p(|p| {
                p.static_nodes()
                    .static_render("The initial value is ")
                    .static_render(self.value.get())
                    .line_break()
                    .static_render("The rate is ")
                    .static_render(self.rate)
                    ;
            })
            .static_render("Value: ")
            .update_render(&self.value)
            .line_break()
            .static_render("Value / self.rate = ")
            // Unfortunately, Rust fail inference types for this closure
            .update_render(self.value.map(|state: &Self, value: &i32| value / state.rate))
            .line_break()
            .static_render("Value * self.rate = ")
            .update_render(self.value.map(|state: &Self, value: &i32| value * state.rate))
            .line_break()
            .static_render("Value * self.rate  * 2 = ")
            .update_render(self.value.map2(|state: &Self, value: &i32| value * state.rate * 2))
            .line_break()
            .static_render(Button("-", comp.handler_mut(State::decrement)))
            .static_render(Button("+", comp.handler_mut(State::increment)))
            .line_break()
            .static_render("This value will be never updated if the update method return `spair::ShouldRender::No`: ")
            .update_render(self.value.get())
            ;
    }
}

struct Button<H>(&'static str, H);
impl<H: spair::Click> spair::StaticRender<State> for Button<H> {
    fn render(self, nodes: spair::StaticNodes<State>) {
        nodes.button(|b| {
            b.static_attributes()
                .on_click(self.1)
                .static_nodes()
                .static_render(self.0);
        });
    }
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
    State::mount_to("root");
}
