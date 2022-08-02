use spair::prelude::*;

pub struct ChildState {
    callback: spair::Callback,
    _callback_arg: spair::CallbackArg<i32>,
    value: i32,
}

impl ChildState {
    pub fn new(callback: spair::Callback, _callback_arg: spair::CallbackArg<i32>) -> Self {
        Self {
            callback,
            _callback_arg,
            value: 42,
        }
    }

    pub fn value(&self) -> i32 {
        self.value
    }

    pub fn set_value(&mut self, value: i32) {
        self.value = value;
    }

    fn increment(&mut self) {
        self.value += 1;
        self.update_related_component()
    }

    fn decrement(&mut self) {
        self.value -= 1;
        self.update_related_component()
    }

    fn update_related_component(&self) {
        if self.value % 5 == 0 {
            self.callback.queue();
            // or
            // self._callback_arg.queue(self.value);
        }
    }
}

impl spair::Component for ChildState {
    type Routes = ();

    fn render(&self, element: spair::Element<Self>) {
        let comp = element.comp();
        element
            .static_nodes()
            .p(|p| {
                p.static_render(
                    "This counter is in a child-component,\
                    the parent component will be notified every \
                    time the value is divisible by five.",
                );
            })
            .static_render(super::Button("-", comp.handler_mut(ChildState::decrement)))
            .update_render(self.value)
            .static_render(super::Button("+", comp.handler_mut(ChildState::increment)));
    }
}

impl spair::AsChildComp for ChildState {
    type Properties = (spair::Callback, spair::CallbackArg<i32>);
    fn init(_comp: &spair::Comp<Self>, props: Self::Properties) -> Self {
        Self::new(props.0, props.1)
    }
}
