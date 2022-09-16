use crate::{
    component::Component,
    dom::WsElement,
    queue_render::value::{MapValue, QrVal},
    render::{base::ElementUpdaterMut, html::HtmlElementUpdater},
};

// These methods don't have to be implemented on HtmlElementUpdater because
// they are for queue-render. But their equivalent methods (for incremental
// render) need to be on HtmlElementUpdater, so these methods need to be on
// HtmlElementUpdater, too.
impl<'a, C: Component> HtmlElementUpdater<'a, C> {
    pub fn qr_property<T: 'static>(
        &self,
        fn_update: impl Fn(&WsElement, &T) + 'static,
        value: &QrVal<T>,
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
