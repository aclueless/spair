use spair::prelude::*;

pub struct ChildState {
    props: ChildProps,
    value: i32,
}

pub struct ChildProps {
    pub title: &'static str,
    pub callback_arg: spair::CallbackArg<i32>,
}

impl ChildState {
    pub fn new(props: ChildProps) -> Self {
        Self { props, value: 42 }
    }

    pub fn set_value(&mut self, value: i32) {
        self.value = value;
    }

    fn increment(&mut self) {
        self.value += 1;
        self.update_parent_component()
    }

    fn decrement(&mut self) {
        self.value -= 1;
        self.update_parent_component()
    }

    fn update_parent_component(&self) {
        if self.value % 5 == 0 {
            self.props.callback_arg.queue(self.value);
        }
    }
}

impl spair::Component for ChildState {
    type Routes = ();

    fn render(&self, element: spair::Element<Self>) {
        let comp = element.comp();
        element
            .static_nodes()
            .div(|d| d.rstatic(self.props.title).done())
            .p(|p| {
                p.rstatic(
                    "This counter is in a child-component, \
                    the parent component will be notified every \
                    time the value is divisible by five.",
                );
            })
            .rstatic(super::Button("-", comp.handler_mut(ChildState::decrement)))
            .rupdate(self.value)
            .rstatic(super::Button("+", comp.handler_mut(ChildState::increment)));
    }
}

impl spair::AsChildComp for ChildState {
    const ROOT_ELEMENT_TAG: spair::TagName = spair::TagName::Html(spair::HtmlTag("div"));
    type Properties = ChildProps;
    fn init(_comp: &spair::Comp<Self>, props: Self::Properties) -> Self {
        Self::new(props)
    }
}
