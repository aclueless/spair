#[macro_export]
macro_rules! set_arm {
    ( $match_if:ident $(,)? ) => {
        $match_if.render_on_arm_index({
            struct Index;
            ::core::any::TypeId::of::<Index>()
        })
    };
}

#[cfg(test)]
macro_rules! make_a_test_component {
    ($RenderType:ident) => {
        use $crate::prelude::*;

        struct TestComponent($RenderType);

        impl TestComponent {
            fn update(&mut self, rt: $RenderType) {
                self.0 = rt;
            }
        }

        impl $crate::Component for TestComponent {
            type Routes = ();
            fn render(&self, element: $crate::Element<Self>) {
                element.rupdate(&self.0);
            }
        }

        impl $crate::Application for TestComponent {
            fn init(_comp: &$crate::Comp<Self>) -> Self {
                TestComponent($RenderType::new())
            }
        }

        struct Test {
            root: crate::dom::Element,
            rc_comp: $crate::component::RcComp<TestComponent>,
            callback: $crate::CallbackArg<$RenderType>,
        }

        impl Test {
            fn set_up() -> Test {
                let root = crate::dom::Element::new_ns($crate::HtmlTag("div"));
                let rc_comp =
                    $crate::application::mount_to_element(root.ws_element().clone().into_inner());
                let callback = rc_comp.comp().callback_arg_mut(TestComponent::update);
                Self {
                    root,
                    rc_comp,
                    callback,
                }
            }

            fn update(&self, value: $RenderType) {
                self.callback.call(value);
            }

            fn execute_on_nodes<T>(&self, func: impl Fn(&[$crate::dom::Node]) -> T) -> T {
                let comp_instance = self.rc_comp.comp_instance();
                let nodes_vec = comp_instance.root_element().nodes().nodes_vec();
                func(nodes_vec)
            }

            fn text_content(&self) -> Option<String> {
                self.root.ws_element().ws_node().text_content()
            }
        }
    };
}
