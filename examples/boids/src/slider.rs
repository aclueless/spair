use spair::prelude::*;

pub struct SliderProps {
    pub id: usize,
    pub label: &'static str,
    pub callback: spair::CallbackArg<f64>,
    pub precision: Option<usize>,
    pub percentage: bool,
    pub min: f64,
    pub max: f64,
    pub step: Option<f64>,
}

impl SliderProps {
    pub fn new(id: usize, callback: spair::CallbackArg<f64>) -> Self {
        Self {
            id,
            label: "",
            callback,
            precision: None,
            percentage: false,
            min: 0.0,
            max: 0.0,
            step: None,
        }
    }

    // The Yew's version don't set this value?
    // pub fn precision(mut self, v: usize) -> Self {
    //     self.precision = Some(v);
    //     self
    // }

    pub fn label(mut self, v: &'static str) -> Self {
        self.label = v;
        self
    }

    pub fn percentage(mut self, v: bool) -> Self {
        self.percentage = v;
        self
    }

    pub fn min(mut self, v: f64) -> Self {
        self.min = v;
        self
    }

    pub fn max(mut self, v: f64) -> Self {
        self.max = v;
        self
    }

    pub fn step(mut self, v: f64) -> Self {
        self.step = Some(v);
        self
    }
}

pub struct Slider {
    props: SliderProps,
    pub value: f64,
}

impl Slider {
    fn value_changed(&self, value: f64) {
        self.props.callback.call_or_queue(value);
    }

    pub fn update_value(&mut self, value: f64) {
        self.value = value;
    }
}

impl spair::Component for Slider {
    type Routes = ();

    fn render(&self, element: spair::Element<Self>) {
        let precision = self
            .props
            .precision
            .unwrap_or_else(|| usize::from(self.props.percentage));

        let display_value = if self.props.percentage {
            format!("{:.p$}%", 100.0 * self.value, p = precision)
        } else {
            format!("{:.p$}", self.value, p = precision)
        };

        let id = format!("slider-{}", self.props.id);
        let step = self.props.step.unwrap_or_else(|| {
            let p = if self.props.percentage {
                precision + 2
            } else {
                precision
            };
            10f64.powi(-(p as i32))
        });

        let comp = element.comp();

        element
            .class("slider")
            .update_nodes()
            .label(|l| {
                l.r#for(&id)
                    .class("slider__label")
                    .rupdate(self.props.label);
            })
            .input(|i| {
                i.input_type(spair::InputType::Range)
                    .id(&id)
                    .class("slider__input")
                    .min(self.props.min)
                    .max(self.props.max)
                    .step(step)
                    .value(self.value.to_string())
                    .on_input(comp.handler_arg_mut(|state, input: spair::InputEvent| {
                        if let Some(input) = input.current_target_as_input_element() {
                            state.value_changed(input.value_as_number())
                        }
                    }));
            })
            .span(|s| {
                s.class("slider__value").rupdate(display_value);
            });
    }
}

impl spair::AsChildComp for Slider {
    const ROOT_ELEMENT_TAG: spair::TagName = spair::TagName::Html(spair::HtmlTag("div"));
    type Properties = SliderProps;
    fn init(_: &spair::Comp<Self>, props: Self::Properties) -> Self {
        Self { props, value: 0.0 }
    }
}
