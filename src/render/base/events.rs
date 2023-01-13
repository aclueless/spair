use super::MethodsForEvents;

impl<'er, C: crate::component::Component, T> StateHelperMethods<'er, C> for T where
    T: MethodsForEvents<'er, C>
{
}

pub trait StateHelperMethods<'er, C: crate::component::Component>:
    MethodsForEvents<'er, C>
{
    fn on_input_value(
        self,
        comp: &crate::Comp<C>,
        updater: impl Fn(&mut C, String) + 'static,
    ) -> Self {
        self.on_input(
            comp.handler_arg_mut(move |state, event: crate::events::InputEvent| {
                if let Some(value) = event.current_target_as_input_element().map(|i| i.value()) {
                    updater(state, value);
                }
            }),
        )
    }
}
