pub trait TextRender<C: crate::component::Component> {
    fn render(self, nodes: &mut super::NodesUpdater<C>);
}

macro_rules! impl_text_render_ref {
    ($($type:ty)+) => {
        $(
            impl<C: crate::component::Component> TextRender<C> for $type {
                fn render(self, nodes: &mut super::NodesUpdater<C>) {
                    nodes.update_text(self);
                }
            }
        )+
    };
}

impl_text_render_ref! {
    i8 i16 i32 i64 isize u8 u16 u32 u64 usize f32 f64 bool char &str &String
}

impl<C: crate::component::Component> TextRender<C> for String {
    fn render(self, nodes: &mut super::NodesUpdater<C>) {
        nodes.update_text(self);
    }
}

#[cfg(test)]
mod tests {
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn update_text_static_text() {
        make_a_test_component! {
            type: u32;
            init: 42;
            render_fn: fn render(&self, element: crate::Element<Self>) {
                element.update_text(self.0).static_text(" ").static_text(self.0);
            }
        }

        let test = Test::set_up();
        assert_eq!(Some("42 42"), test.text_content().as_deref());

        test.update(44);
        assert_eq!(Some("44 42"), test.text_content().as_deref());
    }
}
