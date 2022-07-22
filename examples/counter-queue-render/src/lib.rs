use spair::prelude::*;

struct State {
    factor: i32,
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
            factor: 2,
            value: 4.into(),
        })
    }

    fn default_should_render() -> spair::ShouldRender {
        spair::ShouldRender::Yes
    }

    fn render(&self, element: spair::Element<Self>) {
        let comp = element.comp();
        element
            .static_nodes()
            .p(|p| {
                p.static_nodes()
                    .r#static("The initial value is ")
                    .r#static(self.value.get())
                    .line_break()
                    .r#static("The factor is ")
                    .r#static(self.factor)
                    ;
            })
            .r#static("Value: ")
            .render(&self.value)
            // .line_break()
            // .r#static("Value / self.factor = ")
            // // Unfortunately, Rust fail inference types for this closure
            // .render(self.value.map(|state: &Self, value: &i32| value / state.factor))
            // .line_break()
            // .r#static("Value * self.factor = ")
            // .render(self.value.map(|state: &Self, value: &i32| value * state.factor))
            .line_break()
            .r#static(Button("-", comp.handler_mut(State::decrement)))
            .r#static(Button("+", comp.handler_mut(State::increment)))
            // .line_break()
            // .r#static("This value will be never updated if the update method return `spair::ShouldRender::No`: ")
            // .render(self.value.get())
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
                .r#static(self.0);
        });
    }
}

impl spair::Application for State {
    fn init(_: &spair::Comp<Self>) -> Self {
        Self {
            factor: 3,
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
