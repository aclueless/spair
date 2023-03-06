use super::ElementUpdater;
use crate::component::Component;

#[cfg(feature = "queue-render")]
use crate::queue_render::val::{QrVal, QrValMap, QrValMapWithState};

make_traits_for_attribute_values! {
    BoolAttributeValue {
    //          Incremental render      Queue renders
        bool:   YES                     YES,
    }
    I32AttributeValue {
        i32:    YES                     YES,
    }
    U32AttributeValue {
        u32:    YES                     YES,
    }
    F64AttributeValue {
        f64:    YES                     YES,
    }
    AttributeMinMax {
        f64:    YES                     YES,
        String: YES                     YES,
        &str:   YES                     NO,
    }
    StringAttributeValue {
        String: YES                     YES,
        &str:   YES                     NO,
    }
}

impl<C: Component> AttributeMinMax<C> for &String {
    fn render<'a>(
        self,
        name: &'static str,
        mut element: impl crate::render::base::ElementUpdaterMut<'a, C>,
    ) {
        element.element_updater_mut().attribute(name, self.as_str());
    }
}

impl<C: Component> StringAttributeValue<C> for &String {
    fn render<'a>(
        self,
        name: &'static str,
        mut element: impl crate::render::base::ElementUpdaterMut<'a, C>,
    ) {
        element.element_updater_mut().attribute(name, self.as_str());
    }
}

pub trait Class<C: Component> {
    fn render(self, element: &mut ElementUpdater<C>);
}

impl<C: Component> Class<C> for &str {
    fn render(self, element: &mut ElementUpdater<C>) {
        element.class(self);
    }
}

impl<C: Component> Class<C> for String {
    fn render(self, element: &mut ElementUpdater<C>) {
        element.class(&self);
    }
}

#[cfg(feature = "queue-render")]
impl<C: Component> Class<C> for &QrVal<String> {
    fn render(self, element: &mut ElementUpdater<C>) {
        element.qr_class(self);
    }
}

#[cfg(feature = "queue-render")]
impl<C: Component, T: 'static> Class<C> for QrValMap<T, String> {
    fn render(self, element: &mut ElementUpdater<C>) {
        element.qrm_class(self);
    }
}

#[cfg(feature = "queue-render")]
impl<C: Component, T: 'static> Class<C> for QrValMap<T, &'static str> {
    fn render(self, element: &mut ElementUpdater<C>) {
        element.qrm_str_class(self);
    }
}

#[cfg(feature = "queue-render")]
impl<C: Component, T: 'static> Class<C> for QrValMapWithState<C, T, String> {
    fn render(self, element: &mut ElementUpdater<C>) {
        element.qrmws_class(self);
    }
}

#[cfg(feature = "queue-render")]
impl<C: Component, T: 'static> Class<C> for QrValMapWithState<C, T, &'static str> {
    fn render(self, element: &mut ElementUpdater<C>) {
        element.qrmws_str_class(self);
    }
}
