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
    fn render(&self, c: spair::Context<Self>) {
        let (comp, element) = c.into_parts();
        element
            .static_nodes()
            .p(|p| {
                p.static_nodes()
                    .render("The initial value is ")
                    .render(self.value);
            })
            .button(|b| button(b, "-", comp.handler(State::decrement)))
            .render(self.value)
            .button(|b| button(b, "+", comp.handler(State::increment)));
    }
}

fn button<H: spair::Click>(b: spair::Element<State>, text: &str, h: H) {
    b.static_attributes()
        .on_click(h)
        .static_nodes()
        .r#static(text);
}

#[wasm_bindgen(start)]
pub fn start_counter() {
    wasm_logger::init(wasm_logger::Config::default());
    let state = State { value: 42 };
    spair::application::start(state, "root");
}
