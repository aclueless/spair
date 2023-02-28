use crate::{
    component::Component,
    queue_render::{
        dom::{QrTextNodeMap, QrTextNodeMapWithState},
        val::{QrVal, QrValMap, QrValMapWithState},
    },
    render::base::{NodesUpdater, TextRender},
};

impl<C, T> TextRender<C> for &QrVal<T>
where
    C: Component,
    T: 'static + ToString,
{
    fn render(self, nodes: &mut NodesUpdater<C>) {
        if let Some(text_node) = nodes.create_qr_text_node() {
            match self.content().try_borrow_mut() {
                Ok(mut this) => {
                    text_node.update_text(&this.value().to_string());
                    this.add_render(Box::new(text_node));
                }
                Err(e) => log::error!("{}", e),
            }
        }
    }
}

impl<C, T, U> TextRender<C> for QrValMap<T, U>
where
    C: Component,
    T: 'static + ToString,
    U: 'static + ToString,
{
    fn render(self, nodes: &mut NodesUpdater<C>) {
        if let Some(text_node) = nodes.create_qr_text_node() {
            let (value, fn_map) = self.into_parts();
            let map_node = QrTextNodeMap::new(text_node, fn_map);
            match value.content().try_borrow_mut() {
                Ok(mut this) => {
                    let u = map_node.map(this.value());
                    map_node.update_text(&u.to_string());
                    this.add_render(Box::new(map_node));
                }
                Err(e) => log::error!("{}", e),
            };
        }
    }
}

impl<C, T, U> TextRender<C> for QrValMapWithState<C, T, U>
where
    C: Component,
    T: 'static + ToString,
    U: 'static + ToString,
{
    fn render(self, nodes: &mut NodesUpdater<C>) {
        let state = nodes.state();
        let comp = nodes.comp();
        if let Some(text_node) = nodes.create_qr_text_node() {
            let (value, fn_map) = self.into_parts();
            let map_node = QrTextNodeMapWithState::new(text_node, comp, fn_map);
            match value.content().try_borrow_mut() {
                Ok(mut this) => {
                    let u = map_node.map_with_state(state, this.value());
                    map_node.update_text(&u.to_string());
                    this.add_render(Box::new(map_node));
                }
                Err(e) => log::error!("{}", e),
            };
        }
    }
}

#[cfg(test)]
mod tests {
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn qr_update_text() {
        make_a_test_component! {
            type: crate::QrVal<u32>;
            init: 42.into();
            render_fn: fn render(&self, element: crate::Element<Self>) {
                element
                    .update_text(&self.0)
                    .static_text(" ")
                    // Currently, render an QrVal with `.static_text` still updates on changes
                    .static_text(&self.0);
            }
        }

        let test = Test::set_up();
        assert_eq!(Some("42 42"), test.text_content().as_deref());

        // Currently, render an QrVal with `.static_text` still updates on changes
        test.update_with(|val| val.set_with(|v| v + 2));
        assert_eq!(Some("44 44"), test.text_content().as_deref());
    }
}
