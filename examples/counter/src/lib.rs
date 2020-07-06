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
    fn with_comp(_: spair::Comp<Self>) -> Option<Self> {
        Some(Self { value: 42 })
    }

    fn render(&self, c: spair::Context<Self>) {
        let (comp, element) = c.into_parts();
        element
            .static_nodes()
            .p(|p| {
                p.static_nodes()
                    .r#static("The initial value is ")
                    .r#static(self.value);
            })
            .r#static(Button("-", comp.handler(State::decrement)))
            .render(self.value)
            .r#static(Button("+", comp.handler(State::increment)));
    }
}

struct Button<H>(&'static str, H);
impl<'a, H: spair::Click> spair::StaticRender<'a, State> for Button<H> {
    fn render(
        self,
        _: &State,
        nodes: spair::StaticNodes<'a, State>,
    ) -> spair::StaticNodes<'a, State> {
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
    State::mount_to("root");
}
