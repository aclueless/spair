use crate::component::Component;

pub trait TextRender<C: Component> {
    fn render(self, nodes: &mut super::NodesUpdater<C>, update_mode: bool);
    #[cfg(feature = "nightly-text-render")]
    fn text(self) -> TextRenderOnElement<C, Self>
    where
        Self: Sized,
    {
        TextRenderOnElement {
            value: self,
            update_mode: true,
            c: std::marker::PhantomData,
        }
    }
    #[cfg(feature = "nightly-text-render")]
    fn static_text(self) -> TextRenderOnElement<C, Self>
    where
        Self: Sized,
    {
        TextRenderOnElement {
            value: self,
            update_mode: false,
            c: std::marker::PhantomData,
        }
    }
}

#[cfg(feature = "nightly-text-render")]
pub struct TextRenderOnElement<C, T> {
    value: T,
    update_mode: bool,
    c: std::marker::PhantomData<C>,
}

macro_rules! impl_text_render_ref {
    ($($type:ty)+) => {
        $(
            impl<C: Component> TextRender<C> for $type {
                fn render(self, nodes: &mut super::NodesUpdater<C>, update_mode: bool) {
                    nodes.update_text(self, update_mode);
                }
            }
        )+
    };
}

impl_text_render_ref! {
    i8 i16 i32 i64 isize u8 u16 u32 u64 usize f32 f64 bool char &str &String
}

impl<C: Component> TextRender<C> for String {
    fn render(self, nodes: &mut super::NodesUpdater<C>, update_mode: bool) {
        nodes.update_text(self, update_mode);
    }
}

impl<C: Component, T> TextRender<C> for Option<T>
where
    T: TextRender<C> + crate::dom::InternalTextRender,
{
    fn render(self, nodes: &mut super::NodesUpdater<C>, update_mode: bool) {
        let mi = nodes.get_match_if_updater();
        match self {
            None => {
                mi.render_on_arm_index(std::any::TypeId::of::<usize>());
            }
            Some(value) => mi
                .render_on_arm_index(std::any::TypeId::of::<isize>())
                .update_text(value, update_mode),
        }
    }
}

#[cfg(feature = "nightly-text-render")]
impl<'a, C: Component, T> FnOnce<(crate::Element<'a, C>,)> for TextRenderOnElement<C, T>
where
    T: TextRender<C>,
{
    type Output = ();
    extern "rust-call" fn call_once(self, (element,): (crate::Element<'a, C>,)) -> Self::Output {
        use super::NodesUpdaterMut;
        let mut nodes: crate::render::html::NodesOwned<C> = element.into();
        self.value
            .render(nodes.nodes_updater_mut(), self.update_mode);
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
                element
                    .update_text(self.0)
                    .static_text(" ")
                    .static_text(self.0)
                    .static_text(" ")
                    .update_text(self.0)
                    .static_text(" Option: ")
                    .update_text(None::<u32>)
                    .update_text(Some(self.0))
                    .static_text(" ")
                    .static_text(Some(self.0));
            }
        }

        let test = Test::set_up();
        assert_eq!(
            Some("42 42 42 Option: 42 42"),
            test.text_content().as_deref()
        );

        test.update(44);
        assert_eq!(
            Some("44 42 44 Option: 44 42"),
            test.text_content().as_deref()
        );

        test.update(43);
        assert_eq!(
            Some("43 42 43 Option: 43 42"),
            test.text_content().as_deref()
        );
    }

    #[cfg(feature = "nightly-text-render")]
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn update_text_static_text_as_fn_once() {
        use super::TextRender;
        make_a_test_component! {
            type: u32;
            init: 42;
            render_fn: fn render(&self, element: crate::Element<Self>) {
                element
                    .div(self.0.text())
                    .static_text(" ")
                    .div(self.0.static_text())
                    .static_text(" ")
                    .div(self.0.text())
                    .static_text(" Option: ")
                    .div(None::<u32>.text())
                    .div(Some(self.0).text())
                    .static_text(" ")
                    .div(Some(self.0).static_text());
            }
        }

        let test = Test::set_up();
        assert_eq!(
            Some("42 42 42 Option: 42 42"),
            test.text_content().as_deref()
        );

        test.update(44);
        assert_eq!(
            Some("44 42 44 Option: 44 42"),
            test.text_content().as_deref()
        );

        test.update(43);
        assert_eq!(
            Some("43 42 43 Option: 43 42"),
            test.text_content().as_deref()
        );
    }
}
