use spair::prelude::*;

pub struct Child {
    value: i32,
    callback_arg: CallbackArg<i32>,
}

impl Child {
    pub fn new(callback_arg: CallbackArg<i32>) -> Self {
        Self {
            value: 42,
            callback_arg,
        }
    }

    pub fn set_value(&mut self, value: i32) {
        self.value = value;
    }

    fn increment(&mut self) {
        self.value += 1;
        self.call_to_parent()
    }

    fn decrement(&mut self) {
        self.value -= 1;
        self.call_to_parent()
    }

    fn call_to_parent(&self) {
        if self.value % 5 == 0 {
            self.callback_arg.call(self.value);
        }
    }
}

#[impl_component]
impl Child {
    fn create(cc: &Context<Self>) {}
    fn update(uc: &Context<Self>) {}
    fn view() {
        div(
            text("In child component: "),
            button(
                on_click = cc.comp.callback_arg(|state, _| state.decrement()),
                text("-"),
            ),
            text(uc.state.value),
            button(
                on_click = cc.comp.callback_arg(|state, _| state.increment()),
                text("+"),
            ),
        )
    }
}
