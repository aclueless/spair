// Based on https://codepen.io/mohebifar/pen/KwdeMz
use spair::prelude::*;

struct Clock {
    time: js_sys::Date,
    comp: spair::Comp<Self>,
    clock_closure: Option<gloo_timers::callback::Interval>,
}

impl Clock {
    fn start_clock(&mut self) {
        let cb = self.comp.callback_mut(Self::update_clock);
        self.clock_closure = Some(gloo_timers::callback::Interval::new(1000, move || {
            cb.call_or_queue()
        }));
    }

    fn update_clock(&mut self) {
        self.time = js_sys::Date::new_0();
    }
}

impl spair::Component for Clock {
    type Routes = ();
    fn render(&self, element: spair::Element<Self>) {
        let seconds_angle = 360.0 * self.time.get_seconds() as f64 / 60.0;
        let minutes_angle = 360.0 * self.time.get_minutes() as f64 / 60.0 + seconds_angle / 60.0;
        let hours_angle = (360.0 * self.time.get_hours() as f64 + minutes_angle) / 12.0;

        element.svg(|s| {
            s.view_box("0 0 200 200")
                .width(400.0)
                .height(400.0)
                .static_nodes()
                .filter(|f| {
                    f.id("innerShadow")
                        .x("-20%")
                        .y("-20%")
                        .width("140%")
                        .height("140%")
                        .fe_gaussian_blur(|b| {
                            b.r#in("SourceGraphic").std_deviation(3.0).result("blue");
                        })
                        .fe_offset(|o| {
                            o.r#in("blur").dx(2.5).dy(2.5);
                        });
                })
                .g(|g| {
                    g.circle(|c| {
                        c.id("shadow")
                            .style("fill:rgba(0,0,0,0.4)")
                            .cx(97.0)
                            .cy(100.0)
                            .r(87.0)
                            .filter_attr("url(#innerShadow)");
                    })
                    .circle(|c| {
                        c.id("circle")
                            .style("stroke: #FFFFFF; stroke-width: 12px; fill:#20B7AF")
                            .cx(100.0)
                            .cy(100.0)
                            .r(80.0);
                    });
                })
                .g(|g| {
                    g.list_clone(1..=60, "g", |n, g| {
                        let degree = 360.0 * n as f64 / 60.0;
                        if n % 5 == 0 {
                            let length = 58.0;
                            let dr = degree.to_radians();
                            let dx = dr.sin() * length;
                            let dy = -dr.cos() * length;
                            g.rfn(|nodes| {
                                render_stick(
                                    Stick {
                                        width: 2,
                                        y1: 29,
                                        y2: 32,
                                        angle: degree,
                                    },
                                    nodes,
                                )
                            })
                            .text(|t| {
                                t.x(100.0)
                                    .y(101.0)
                                    .text_anchor("middle")
                                    .dominant_baseline("middle")
                                    .transform(format!("translate({dx} {dy})"))
                                    .update_text(n / 5)
                                    .done()
                            });
                        } else {
                            g.rfn(|nodes| {
                                render_stick(
                                    Stick {
                                        width: 1,
                                        y1: 30,
                                        y2: 32,
                                        angle: degree,
                                    },
                                    nodes,
                                )
                            });
                        }
                    });
                })
                .update_nodes()
                .g(|g| {
                    g.rfn(|nodes| {
                        render_hand(
                            Hand {
                                width: 4,
                                y2: 55,
                                angle: hours_angle,
                            },
                            nodes,
                        )
                    })
                    .rfn(|nodes| {
                        render_hand(
                            Hand {
                                width: 2,
                                y2: 40,
                                angle: minutes_angle,
                            },
                            nodes,
                        )
                    })
                    .rfn(|nodes| {
                        render_hand(
                            Hand {
                                width: 1,
                                y2: 30,
                                angle: seconds_angle,
                            },
                            nodes,
                        )
                    });
                })
                .static_nodes()
                .circle(|c| {
                    c.id("center")
                        .cx(100.0)
                        .cy(100.0)
                        .r(3.0)
                        .style("fill:#128A86; stroke: #C1EFED; stroke-width: 2px;");
                });
        });
    }
}

struct Stick {
    width: u8,
    y1: u8,
    y2: u8,
    angle: f64,
}

fn render_stick(stick: Stick, nodes: spair::SvgNodes<Clock>) {
    nodes.line(|l| {
        l.transform(format!("rotate({} 100 100)", stick.angle))
            .x1(100.0)
            .y1(stick.y1 as f64)
            .x2(100.0)
            .y2(stick.y2 as f64)
            .stroke("white")
            .stroke_width(stick.width as f64);
    });
}

struct Hand {
    width: u8,
    y2: u8,
    angle: f64,
}

fn render_hand(hand: Hand, nodes: spair::SvgNodes<Clock>) {
    nodes.line(|l| {
        l.transform(format!("rotate({} 100 100)", hand.angle))
            .static_attributes()
            .x1(100.0)
            .y1(100.0)
            .x2(100.0)
            .y2(hand.y2 as f64)
            .stroke("white")
            .stroke_width(hand.width as f64);
    });
}

impl spair::Application for Clock {
    fn init(comp: &spair::Comp<Self>) -> Self {
        let mut s = Self {
            time: js_sys::Date::new_0(),
            comp: comp.clone(),
            clock_closure: None,
        };
        s.start_clock();
        s
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    //wasm_logger::init(wasm_logger::Config::default());
    Clock::mount_to_body();
}
