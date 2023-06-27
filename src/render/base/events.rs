use super::MethodsForEvents;

impl<'updater, C: crate::component::Component, T> StateHelperMethods<'updater, C> for T where
    T: MethodsForEvents<'updater, C>
{
}

pub trait StateHelperMethods<'updater, C: crate::component::Component>:
    MethodsForEvents<'updater, C>
{
    fn on_input_value(
        self,
        comp: &crate::Comp<C>,
        updater: impl Fn(&mut C, String) + 'static,
    ) -> Self {
        self.on_input(
            comp.handler_arg_mut(move |state, event: crate::events::InputEvent| {
                if let Some(value) = event
                    .current_target()
                    .into_input_element()
                    .map(|i| i.0.value())
                {
                    updater(state, value);
                }
            }),
        )
    }
}
