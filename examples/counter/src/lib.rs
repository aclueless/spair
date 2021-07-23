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
                    .r#static("The initial value is ")
                    .r#static(self.value);
            })
            .r#static(Button("-", comp.handler_mut(State::decrement)))
            .render(self.value)
            .r#static(Button("+", comp.handler_mut(State::increment)));
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
    fn with_comp(_: spair::Comp<Self>) -> Self {
        Self { value: 42 }
    }
}

#[wasm_bindgen(start)]
pub fn start_counter() {
    State::mount_to("root");
}
