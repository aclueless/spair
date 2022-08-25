use crate::{
    component::Component,
    dom::WsElement,
    queue_render::{MapValue, Value},
    render::{base::ElementRenderMut, html::HtmlElementRender},
};

// These methods don't have to be implemented on HtmlElementRender because
// they are for queue-render. But their equivalent methods (for incremental
// render) need to be on HtmlElementRender, so these methods need to be on
// HtmlElementRender, too.
impl<'a, C: Component> HtmlElementRender<'a, C> {
    pub fn qr_property<T: 'static>(
        &self,
        fn_update: impl Fn(&WsElement, &T) + 'static,
        value: &Value<T>,
    ) {
        self.element_render().qr_property(fn_update, value)
    }

    pub fn qrm_property<T: 'static, U: 'static>(
        &self,
        fn_update: impl Fn(&WsElement, &U) + 'static,
        value: MapValue<C, T, U>,
    ) {
        self.element_render().qrm_property(fn_update, value)
    }
}
