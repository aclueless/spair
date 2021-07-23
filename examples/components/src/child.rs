use spair::prelude::*;

pub struct ChildState {
    parent_comp: spair::Comp<super::State>,
    value: i32,
}

impl ChildState {
    pub fn new(parent_comp: spair::Comp<super::State>) -> Self {
        Self {
            parent_comp,
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
            spair::update_component(
                self.parent_comp
                    .callback_mut(super::State::child_value_is_divisible_by_five),
            );
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
                p.r#static(
                    "This counter is in a child-component,\
                    the parent component will be notified every \
                    time the value is divisible by five.",
                );
            })
            .r#static(super::Button("-", comp.handler_mut(ChildState::decrement)))
            .render(self.value)
            .r#static(super::Button("+", comp.handler_mut(ChildState::increment)));
    }
}
