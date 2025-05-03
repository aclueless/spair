use spair::prelude::*;

mod child;
use child::Child;

pub struct State {
    value: i32,
    value_from_child: Option<i32>,
    child_comp: RcComp<Child>,
}

impl State {
    fn increment(&mut self) {
        self.value += 1;
    }

    fn decrement(&mut self) {
        self.value -= 1;
    }

    pub fn receive_value_from_child(&mut self, value: i32) {
        self.value_from_child = Some(value);
    }

    fn send_value_to_child(&mut self) {
        let value = self.value;
        self.child_comp
            .comp()
            .callback_arg(Child::set_value)
            .call(value);
    }
}

#[component_for]
impl State {
    fn create(cc: &Context<Self>) {}
    fn update(uc: &Context<Self>) {}
    fn view() {
        div(
            replace_at_element_id = "root",
            text("Interaction between components"),
            hr(),
            div(
                text("In root component: "),
                button(
                    on_click = cc.comp.callback_arg(|state, _| state.decrement()),
                    text("-"),
                ),
                text(uc.state.value),
                button(
                    on_click = cc.comp.callback_arg(|state, _| state.increment()),
                    text("+"),
                ),
                button(
                    on_click = cc.comp.callback_arg(|state, _| state.send_value_to_child()),
                    text("Send value to child component"),
                ),
            ),
            div(text(
                "Value received from child component: ",
                uc.state.value_from_child.or_default("not het"),
            )),
            hr(),
            ws.element(cc.state.child_comp.root_element()),
        )
    }
}

pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    spair::start_app(|app_comp| State {
        value: 42,
        value_from_child: None,
        child_comp: RcComp::new(|_child_comp| {
            Child::new(app_comp.callback_arg(State::receive_value_from_child))
        }),
    });
}
