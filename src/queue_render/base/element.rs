use crate::{
    component::Component,
    dom::{ElementStatus, WsElement},
    queue_render::val::{QrVal, QrValMapWithState, QueueRender},
    render::base::ElementUpdater,
};

use super::{
    AttributeUpdater, QrClass, QrClassMap, QrNormalAttribute, QrNormalAttributeMap, QrProperty,
    QrPropertyMap,
};

impl<'a, C: Component> ElementUpdater<'a, C> {
    pub fn qr_attribute<T: 'static + AttributeUpdater>(
        &self,
        name: &'static str,
        value: &QrVal<T>,
    ) {
        if self.status() == ElementStatus::Existing {
            return;
        }
        let element = self.element().ws_element().clone();
        let unmounted = self.element().unmounted();
        let mut q = QrNormalAttribute::new(element, unmounted, name);

        match value.content().try_borrow_mut() {
            Ok(mut this) => {
                q.render(this.value());
                this.add_render(Box::new(q));
            }
            Err(e) => log::error!("{}", e),
        }
    }

    // qrmws = queue render map with state
    pub fn qrmws_attribute<T: 'static, U: 'static + AttributeUpdater>(
        &self,
        name: &'static str,
        value: QrValMapWithState<C, T, U>,
    ) {
        if self.status() == ElementStatus::Existing {
            return;
        }

        let element = self.element().ws_element().clone();
        let unmounted = self.element().unmounted();
        let mut q = QrNormalAttribute::new(element, unmounted, name);

        let state = self.state();
        let (value, fn_map) = value.into_parts();
        match value.content().try_borrow_mut() {
            Ok(mut this) => {
                let u = (fn_map)(state, this.value());
                q.render(&u);
                let q = QrNormalAttributeMap::new(q, self.comp(), fn_map);
                this.add_render(Box::new(q));
            }
            Err(e) => log::error!("{}", e),
        };
    }

    pub fn qr_property<T: 'static>(
        &self,
        fn_update: impl Fn(&WsElement, &T) + 'static,
        value: &QrVal<T>,
    ) {
        if self.status() == ElementStatus::Existing {
            return;
        }
        let element = self.element().ws_element().clone();
        let unmounted = self.element().unmounted();
        let mut q = QrProperty::new(element, unmounted, Box::new(fn_update));

        match value.content().try_borrow_mut() {
            Ok(mut this) => {
                q.render(this.value());
                this.add_render(Box::new(q));
            }
            Err(e) => log::error!("{}", e),
        }
    }

    // qrmws = queue render map with state
    pub fn qrmws_property<T: 'static, U: 'static>(
        &self,
        fn_update: impl Fn(&WsElement, &U) + 'static,
        value: QrValMapWithState<C, T, U>,
    ) {
        if self.status() == ElementStatus::Existing {
            return;
        }
        let element = self.element().ws_element().clone();
        let unmounted = self.element().unmounted();
        let mut q = QrProperty::new(element, unmounted, Box::new(fn_update));

        let state = self.state();
        let (value, fn_map) = value.into_parts();
        match value.content().try_borrow_mut() {
            Ok(mut this) => {
                let u = (fn_map)(state, this.value());
                q.render(&u);
                this.add_render(Box::new(QrPropertyMap::new(q, self.comp(), fn_map)));
            }
            Err(e) => log::error!("{}", e),
        };
    }

    pub fn qr_class(&self, value: &QrVal<String>) {
        if self.status() == ElementStatus::Existing {
            return;
        }
        let element = self.element().ws_element().clone();
        let unmounted = self.element().unmounted();
        let mut q = QrClass::new(element, unmounted);

        match value.content().try_borrow_mut() {
            Ok(mut this) => {
                q.render(this.value());
                this.add_render(Box::new(q));
            }
            Err(e) => log::error!("{}", e),
        }
    }

    pub fn qrmws_class<T: 'static>(&self, value: QrValMapWithState<C, T, String>) {
        if self.status() == ElementStatus::Existing {
            return;
        }
        let element = self.element().ws_element().clone();
        let unmounted = self.element().unmounted();
        let mut q = QrClass::new(element, unmounted);

        let state = self.state();
        let (value, fn_map) = value.into_parts();
        match value.content().try_borrow_mut() {
            Ok(mut this) => {
                let u = (fn_map)(state, this.value());
                q.render(&u);
                this.add_render(Box::new(QrClassMap::new(q, self.comp(), fn_map)));
            }
            Err(e) => log::error!("{}", e),
        };
    }

    pub fn qrmws_str_class<T: 'static>(&self, value: QrValMapWithState<C, T, &'static str>) {
        if self.status() == ElementStatus::Existing {
            return;
        }
        let element = self.element().ws_element().clone();
        let unmounted = self.element().unmounted();
        let mut q = QrClass::new(element, unmounted);

        let state = self.state();
        let (value, fn_map) = value.into_parts();
        match value.content().try_borrow_mut() {
            Ok(mut this) => {
                let u = (fn_map)(state, this.value());
                q.render(&u);
                this.add_render(Box::new(QrClassMap::new(q, self.comp(), fn_map)));
            }
            Err(e) => log::error!("{}", e),
        };
    }

    // pub fn qr_list_render(&mut self, mode: ListElementCreation, tag: &'a str) -> ListRender<C> {
    //     let (parent, nodes) = self.element.ws_node_and_nodes_mut();
    //     ListRender::new(
    //         self.comp,
    //         self.state,
    //         nodes,
    //         tag,
    //         parent,
    //         None,
    //         mode.use_template(),
    //     )
    // }
}
