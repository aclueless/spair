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

    fn increment(&mut self) -> spair::Checklist<ChildState> {
        self.value += 1;
        self.update_related_component()
    }

    fn decrement(&mut self) -> spair::Checklist<ChildState> {
        self.value -= 1;
        self.update_related_component()
    }

    fn update_related_component(&self) -> spair::Checklist<ChildState> {
        let mut cl = spair::Checklist::default();
        if self.value % 5 == 0 {
            cl.update_related_component(
                self.parent_comp
                    .callback_child_comps(super::State::child_value_is_divisible_by_five),
            );
        }
        cl
    }
}

impl spair::Component for ChildState {
    type Routes = ();
    type Components = ();
    fn render(&self, c: spair::Context<Self>) {
        let (comp, element) = c.into_comp_element();
        element
            .static_nodes()
            .p(|p| {
                p.static_nodes()
                    .render("This counter is in a child-component, the parent component will be notified every time the value is divisible by five.");
            })
            .r#static(super::Button("-", comp.handler(ChildState::decrement)))
            .render(self.value)
            .r#static(super::Button("+", comp.handler(ChildState::increment)));
    }
}
