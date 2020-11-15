use spair::prelude::*;

struct Clock {
    time: f64,
}

impl spair::Component for Clock {
    type Routes = ();
    fn render(&self, element: spair::Element<Self>) {
        element.div(|d| {
            d.svg(|s| {
                s.view_box("0 0 100 100")
                    .width(400.0)
                    .height(400.0)
                    .circle(|c| c.cx(50.0).cy(50.0).r(50.0).done());
            });
        });
    }
}

impl spair::Application for Clock {
    fn with_comp(_: spair::Comp<Self>) -> Self {
        Self {
            time: js_sys::Date::now(),
        }
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    Clock::mount_to_body();
}
