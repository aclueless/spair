use std::hash::Hash;

use crate::{Component, Context, TemplateElement, WsElement};

pub trait KeyedListItemView<C: Component> {
    type ViewState;
    type Key: Clone + Eq + Hash;
    fn template_string() -> &'static str;
    fn get_key(&self) -> &Self::Key;
    fn key_from_view_state(state: &Self::ViewState) -> &Self::Key;
    fn create(template: &TemplateElement, cdata: &Self, ccontext: &Context<C>) -> Self::ViewState;
    fn update(view_state: &mut Self::ViewState, udata: &Self, ucontext: &Context<C>);
    fn root_element(view_state: &Self::ViewState) -> &WsElement;
}

#[cfg(test)]
pub mod keyed_list_via_trait_tests {
    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::Node;

    use crate::{
        Element, KeyedList, Text,
        test_helper::{self, TestComp, TestDataInterface},
    };

    use super::KeyedListItemView;

    #[derive(Debug, Clone, Eq, PartialEq)]
    struct TestData(Vec<String>);
    type TestState = TestComp<TestData>;

    impl TestData {
        fn from_strs(strs: &[&str]) -> Self {
            Self(strs.iter().map(|v| v.to_string()).collect())
        }
    }

    pub struct TestDataViewState {
        keyed_list: KeyedList<
            TestState,
            String,
            <String as KeyedListItemView<TestState>>::Key,
            <String as KeyedListItemView<TestState>>::ViewState,
        >,
    }

    pub struct TestItemViewState {
        data: String,
        element: Element,
        text: Text,
    }

    impl KeyedListItemView<TestState> for String {
        type ViewState = TestItemViewState;

        type Key = String;

        fn template_string() -> &'static str {
            "<span>?</span>"
        }

        fn get_key(&self) -> &Self::Key {
            &self
        }

        fn key_from_view_state(view_state: &Self::ViewState) -> &Self::Key {
            &view_state.data
        }

        fn create(
            template: &crate::TemplateElement,
            item_data: &Self,
            _context: &crate::Context<TestState>,
        ) -> Self::ViewState {
            let element = template.create_element(0);
            let text = element.ws_node_ref().first_text();
            TestItemViewState {
                data: item_data.to_string(),
                element,
                text,
            }
        }

        fn update(
            view_state: &mut Self::ViewState,
            item_data: &Self,
            _context: &crate::Context<TestState>,
        ) {
            view_state.text.update(item_data);
        }

        fn root_element(view_state: &Self::ViewState) -> &crate::WsElement {
            &view_state.element
        }
    }

    impl TestDataInterface for TestData {
        type ViewState = TestDataViewState;

        fn init(&self, root: &Element, context: &crate::Context<TestState>) -> Self::ViewState {
            let mut keyed_list = KeyedList::new(
                &root.ws_element(),
                None,
                String::template_string(),
                String::get_key,
                String::key_from_view_state,
                String::create,
                String::update,
                String::root_element,
            );
            keyed_list.update(self.0.iter(), context);
            TestDataViewState { keyed_list }
        }

        fn update(&self, view_state: &mut Self::ViewState, context: &crate::Context<TestState>) {
            view_state.keyed_list.update(self.0.iter(), &context);
        }
    }

    fn collect_text_from_child_nodes(root_node: &Node) -> TestData {
        let mut list = Vec::new();
        let mut maybe_node = root_node.first_child();
        while let Some(node) = maybe_node {
            if let Some(text) = node.text_content() {
                list.push(text);
            }
            maybe_node = node.next_sibling();
        }
        TestData(list)
    }

    #[wasm_bindgen_test]
    fn keyed_list_via_trait() {
        let test = test_helper::Test::set_up(TestData(Vec::new()));
        assert_eq!(Some(""), test.text_content().as_deref());
        let empty = TestData(Vec::new());
        test.update(empty.clone());
        assert_eq!(Some(""), test.text_content().as_deref());
        assert_eq!(
            empty,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );

        let data = TestData::from_strs(&["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"]);
        test.update(data.clone());
        assert_eq!(
            data,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );
        assert_eq!(Some("abcdefghijk"), test.text_content().as_deref());

        // Random shuffle + addition
        let data = TestData::from_strs(&["f", "b", "d", "l", "g", "i", "m", "j", "a", "h", "k"]);
        test.update(data.clone());
        assert_eq!(Some("fbdlgimjahk"), test.text_content().as_deref());
        assert_eq!(
            data,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );

        // Empty the list
        test.update(empty.clone());
        assert_eq!(Some(""), test.text_content().as_deref());
        assert_eq!(
            empty,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );

        // Add back
        let data = TestData::from_strs(&["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"]);
        test.update(data.clone());
        assert_eq!(
            data,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );
        assert_eq!(Some("abcdefghijk"), test.text_content().as_deref());

        // Forward
        let data = TestData::from_strs(&["a", "i", "b", "c", "d", "e", "f", "g", "h", "j", "k"]);
        test.update(data.clone());
        assert_eq!(
            data,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );
        assert_eq!(Some("aibcdefghjk"), test.text_content().as_deref());

        // Backward
        let data = TestData::from_strs(&["a", "i", "c", "d", "e", "f", "g", "h", "b", "j", "k"]);
        test.update(data.clone());
        assert_eq!(
            data,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );
        assert_eq!(Some("aicdefghbjk"), test.text_content().as_deref());

        // Swap
        let data = TestData::from_strs(&["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"]);
        test.update(data.clone());
        assert_eq!(
            data,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );
        assert_eq!(Some("abcdefghijk"), test.text_content().as_deref());

        // Remove middle
        let data = TestData::from_strs(&["a", "b", "c", "d", "i", "j", "k"]);
        test.update(data.clone());
        assert_eq!(
            data,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );
        assert_eq!(Some("abcdijk"), test.text_content().as_deref());

        // Insert middle
        let data = TestData::from_strs(&["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"]);
        test.update(data.clone());
        assert_eq!(
            data,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );
        assert_eq!(Some("abcdefghijk"), test.text_content().as_deref());

        // Remove start
        let data = TestData::from_strs(&["d", "e", "f", "g", "h", "i", "j", "k"]);
        test.update(data.clone());
        assert_eq!(
            data,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );
        assert_eq!(Some("defghijk"), test.text_content().as_deref());

        // Insert start
        let data = TestData::from_strs(&["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"]);
        test.update(data.clone());
        assert_eq!(
            data,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );
        assert_eq!(Some("abcdefghijk"), test.text_content().as_deref());

        // Remove end
        let data = TestData::from_strs(&["a", "b", "c", "d", "e", "f", "g", "h"]);
        test.update(data.clone());
        assert_eq!(
            data,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );
        assert_eq!(Some("abcdefgh"), test.text_content().as_deref());

        // Append end
        let data =
            TestData::from_strs(&["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "z"]);
        test.update(data.clone());
        assert_eq!(
            data,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );
        assert_eq!(Some("abcdefghijkz"), test.text_content().as_deref());
    }
}
