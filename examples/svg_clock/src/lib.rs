use spair::prelude::*;

struct Clock {
    time: f64,
}

impl spair::Component for Clock {
    type Routes = ();
    fn render(&self, element: spair::Element<Self>) {
        element._svg(|s| {
            s.view_box("0 0 100 100")
                .width(400.0)
                .height(400.0)
                .circle(|c| {
                    c.cx(50.0)
                        .cy(50.0)
                        .r(49.0)
                        .fill("none")
                        .stroke("#0000ff")
                        .done()
                })
                .render(Line{
                    width: 5.0,
                    height: 30.0,
                    color: "black",
                }).render(Line{
                    width: 3.0,
                    height: 35.0,
                    color: "black",
                }).render(Line{
                    width: 1.0,
                    height: 40.0,
                    color: "gray",
                });
        });
    }
}

struct Line {
    width: f64,
    height: f64,
    color: &'static str,
}

impl spair::SvgRender<Clock> for Line {
    fn render(self, nodes: spair::SvgNodes<Clock>) {
        nodes.line(|l| {
            l.x1(50.0)
                .y1(50.0)
                .x2(0.0)
                .y2(self.height)
                .stroke(self.color)
                .stroke_width(self.width);
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
    wasm_logger::init(wasm_logger::Config::default());
    Clock::mount_to("root");
}
