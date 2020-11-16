// https://codepen.io/mohebifar/pen/KwdeMz
use spair::prelude::*;

struct Clock {
    time: js_sys::Date,
}

impl spair::Component for Clock {
    type Routes = ();
    fn render(&self, element: spair::Element<Self>) {
        let hours_angle =
            360.0 * self.time.get_hours() as f64 / 12.0 + self.time.get_minutes() as f64 / 2.0;
        let minutes_angle = 360.0 * self.time.get_minutes() as f64 / 60.0;
        let seconds_angle = 360.0 * self.time.get_seconds() as f64 / 60.0;

        let h_from = format!("{} 100 100", hours_angle);
        let h_to = format!("{} 100 100", hours_angle + 360.0);
        let m_from = format!("{} 100 100", minutes_angle);
        let m_to = format!("{} 100 100", minutes_angle + 360.0);
        let s_from = format!("{} 100 100", seconds_angle);
        let s_to = format!("{} 100 100", seconds_angle + 360.0);

        element._svg(|s| {
            s.view_box("0 0 200 200")
                .width(400.0)
                .height(400.0)
                .static_nodes()
                .filter(|f| {
                    f.id("innerShadow")
                        .str_attr("x", "-20%")
                        .str_attr("y", "-20%")
                        .str_attr("width", "140%")
                        .str_attr("height", "140%")
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
                .nodes()
                .g(|g| {
                    g.line(|l| {
                        l.static_attributes()
                            .x1(100.0)
                            .y1(100.0)
                            .x2(100.0)
                            .y2(55.0)
                            //.transform("rotate(80 100 100)")
                            .style("stroke-width: 4px; stroke: #fffbf9;")
                            .id("hourhand")
                            .animate_transform(|a| {
                                a.from(&h_from)
                                    .to(&h_to)
                                    .static_attributes()
                                    .attribute_name("transform")
                                    .attribute_type("XML")
                                    .r#type("rotate")
                                    .dur("43200s")
                                    .repeat_count("indefinite");
                            });
                    })
                    .line(|l| {
                        l.static_attributes()
                            .x1(100.0)
                            .y1(100.0)
                            .x2(100.0)
                            .y2(40.0)
                            .style("stroke-width: 2px; stroke: #fdfdfd;")
                            .id("minutehand")
                            .animate_transform(|a| {
                                a.from(&m_from)
                                    .to(&m_to)
                                    .static_attributes()
                                    .attribute_name("transform")
                                    .attribute_type("XML")
                                    .r#type("rotate")
                                    .dur("3600s")
                                    .repeat_count("indefinite");
                            });
                    })
                    .line(|l| {
                        l.static_attributes()
                            .x1(100.0)
                            .y1(100.0)
                            .x2(100.0)
                            .y2(30.0)
                            .style("stroke-width: 1px; stroke: #C1EFED;")
                            .id("secondhand")
                            .animate_transform(|a| {
                                a.from(&s_from)
                                    .to(&s_to)
                                    .static_attributes()
                                    .attribute_name("transform")
                                    .attribute_type("XML")
                                    .r#type("rotate")
                                    .dur("60s")
                                    .repeat_count("indefinite");
                            });
                    });
                })
                .static_nodes()
                .g(|g| {
                    g.list_with_render(1..=12, spair::ListElementCreation::Clone, "line", |n, l| {
                        l.x1(100.0)
                            .y1(30.0)
                            .x2(100.0)
                            .y2(40.0)
                            .transform(&format!("rotate({} 100 100)", *n as f64 * 360.0 / 12.0))
                            .stroke("white");
                    })
                })
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

// struct Line {
//     width: f64,
//     height: f64,
//     color: &'static str,
// }

// impl spair::SvgRender<Clock> for Line {
//     fn render(self, nodes: spair::SvgNodes<Clock>) {
//         nodes.line(|l| {
//             l.x1(50.0)
//                 .y1(50.0)
//                 .x2(0.0)
//                 .y2(self.height)
//                 .stroke(self.color)
//                 .stroke_width(self.width);
//         });
//     }
// }

impl spair::Application for Clock {
    fn with_comp(_: spair::Comp<Self>) -> Self {
        Self {
            time: js_sys::Date::new_0(),
        }
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    wasm_logger::init(wasm_logger::Config::default());
    Clock::mount_to("root");
}
