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
    (
        type: $Type:ty;
        init: $init_expr:expr;
        render_fn: $($render_fn:tt)+
    ) => {
        use $crate::prelude::*;

        struct TestComponent($Type);

        impl TestComponent {
            fn update(&mut self, value: $Type) {
                self.0 = value;
            }
        }

        impl $crate::Component for TestComponent {
            type Routes = ();
            $($render_fn)+
        }

        impl $crate::Application for TestComponent {
            fn init(_comp: &$crate::Comp<Self>) -> Self {
                TestComponent($init_expr)
            }
        }

        struct Test {
            root: web_sys::Node,
            rc_comp: $crate::component::RcComp<TestComponent>,
            callback: $crate::CallbackArg<$Type>,
        }

        impl Test {
            fn set_up() -> Test {
                let root = crate::dom::Element::new_ns($crate::HtmlTag("div"));
                let rc_comp =
                    $crate::application::mount_to_element(root.ws_element().clone().into_inner());
                let callback = rc_comp.comp().callback_arg_mut(TestComponent::update);
                Self {
                    root: root.ws_element().ws_node().clone(),
                    rc_comp,
                    callback,
                }
            }

            #[allow(dead_code)]
            fn update(&self, value: $Type) {
                self.callback.call(value);
            }

            #[allow(dead_code)]
            fn update_with(&self, updater: impl Fn(&mut $Type) + 'static) {
                self.rc_comp.comp().callback_mut(move |state| updater(&mut state.0)).call();
            }

            #[allow(dead_code)]
            fn execute_on_nodes<T>(&self, func: impl Fn(&[$crate::dom::Node]) -> T) -> T {
                let comp_instance = self.rc_comp.comp_instance();
                let nodes_vec = comp_instance.root_element().nodes().nodes_vec();
                func(nodes_vec)
                //func(self.root.nodes().nodes_vec())
            }

            #[allow(dead_code)]
            fn text_content(&self) -> Option<String> {
                self.root.text_content()
            }
        }
    };
}
