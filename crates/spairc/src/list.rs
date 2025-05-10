use crate::{Component, Context, TemplateElement, WsElement};

pub struct List<VS> {
    parent_element: WsElement,
    template: TemplateElement,
    end_node_marker_for_partial_list: Option<web_sys::Node>,

    items: Vec<VS>,
}

impl<VS> List<VS> {
    pub fn new(
        parent_element: &WsElement,
        end_node_marker_for_partial_list: Option<web_sys::Node>,
        template_string: &str,
    ) -> Self {
        Self {
            parent_element: parent_element.clone(),
            template: TemplateElement::new(template_string),
            end_node_marker_for_partial_list,

            items: Vec::new(),
        }
    }

    pub fn update<C, I>(
        &mut self,
        item_data: impl Iterator<Item = I>,
        context: &Context<C>,
        create_view_fn: fn(&TemplateElement, &I, &Context<C>) -> VS,
        update_view_fn: fn(&mut VS, &I, &Context<C>),
        get_view_state_root_element_fn: fn(&VS) -> &WsElement,
    ) where
        C: Component + 'static,
    {
        let mut index = 0;
        for item_data in item_data {
            if index >= self.items.len() {
                let mut new_item = (create_view_fn)(&self.template, &item_data, context);
                (update_view_fn)(&mut new_item, &item_data, context);
                self.parent_element.insert_new_node_before_a_node(
                    (get_view_state_root_element_fn)(&new_item),
                    self.end_node_marker_for_partial_list.as_ref(),
                );
                self.items.push(new_item);
            } else {
                let old_item = unsafe { self.items.get_unchecked_mut(index) };
                (update_view_fn)(old_item, &item_data, context);
            }
            index += 1;
        }

        if index >= self.items.len() {
            return;
        }
        if index == 0 && self.end_node_marker_for_partial_list.is_none() {
            self.parent_element.clear_text_content();
            self.items.clear();
        } else {
            for item in self.items.drain(index..) {
                self.parent_element
                    .remove_child((get_view_state_root_element_fn)(&item));
            }
        }
    }
}

#[cfg(test)]
pub mod list_tests {
    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::Node;

    use crate::{
        Element, Text,
        test_helper::{self, TestInterface},
    };

    use super::List;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestData(Vec<String>);

    trait IntoTestData {
        fn into_test_data(self) -> TestData;
    }

    impl IntoTestData for Vec<&'static str> {
        fn into_test_data(self) -> TestData {
            TestData(self.iter().map(|v| v.to_string()).collect())
        }
    }

    type TestComp = crate::test_helper::TestComp<TestData>;

    pub struct TestViewState {
        list: List<TestItemViewState>,
    }

    pub struct TestItemViewState {
        element: Element,
        text: Text,
    }

    const TEMPLATE_STRING: &str = "<span>?</span>";

    impl TestInterface for TestData {
        type ViewState = TestViewState;

        fn init(&self, root: &Element, context: &crate::Context<TestComp>) -> Self::ViewState {
            let list = List::new(&root.ws_element(), None, TEMPLATE_STRING);
            let mut view_state = TestViewState { list };
            self.update(&mut view_state, context);
            view_state
        }

        fn update(&self, view_state: &mut Self::ViewState, context: &crate::Context<TestComp>) {
            view_state.list.update(
                self.0.iter(),
                &context,
                |template, _item_data, _context| {
                    let element = template.create_element(0);
                    let text = element.ws_node_ref().first_text();
                    TestItemViewState { element, text }
                },
                |view_state, item_data, _context| {
                    view_state.text.update(*item_data);
                },
                |vs: &TestItemViewState| &vs.element,
            );
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
    fn non_keyed_list() {
        let test = test_helper::Test::set_up(TestData(Vec::new()));
        assert_eq!(Some(""), test.text_content().as_deref());
        let empty = TestData(Vec::new());
        test.update(empty.clone());
        assert_eq!(Some(""), test.text_content().as_deref());
        assert_eq!(
            empty,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );

        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"].into_test_data();
        test.update(data.clone());
        assert_eq!(
            data,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );
        assert_eq!(Some("abcdefghijk"), test.text_content().as_deref());

        // Random shuffle + addition
        let data = vec!["f", "b", "d", "l", "g", "i", "m", "j", "a", "h", "k"].into_test_data();
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
        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"].into_test_data();
        test.update(data.clone());
        assert_eq!(
            data,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );
        assert_eq!(Some("abcdefghijk"), test.text_content().as_deref());

        // Forward
        let data = vec!["a", "i", "b", "c", "d", "e", "f", "g", "h", "j", "k"].into_test_data();
        test.update(data.clone());
        assert_eq!(
            data,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );
        assert_eq!(Some("aibcdefghjk"), test.text_content().as_deref());

        // Backward
        let data = vec!["a", "i", "c", "d", "e", "f", "g", "h", "b", "j", "k"].into_test_data();
        test.update(data.clone());
        assert_eq!(
            data,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );
        assert_eq!(Some("aicdefghbjk"), test.text_content().as_deref());

        // Swap
        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"].into_test_data();
        test.update(data.clone());
        assert_eq!(
            data,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );
        assert_eq!(Some("abcdefghijk"), test.text_content().as_deref());

        // Remove middle
        let data = vec!["a", "b", "c", "d", "i", "j", "k"].into_test_data();
        test.update(data.clone());
        assert_eq!(
            data,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );
        assert_eq!(Some("abcdijk"), test.text_content().as_deref());

        // Insert middle
        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"].into_test_data();
        test.update(data.clone());
        assert_eq!(
            data,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );
        assert_eq!(Some("abcdefghijk"), test.text_content().as_deref());

        // Remove start
        let data = vec!["d", "e", "f", "g", "h", "i", "j", "k"].into_test_data();
        test.update(data.clone());
        assert_eq!(
            data,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );
        assert_eq!(Some("defghijk"), test.text_content().as_deref());

        // Insert start
        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"].into_test_data();
        test.update(data.clone());
        assert_eq!(
            data,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );
        assert_eq!(Some("abcdefghijk"), test.text_content().as_deref());

        // Remove end
        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h"].into_test_data();
        test.update(data.clone());
        assert_eq!(
            data,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );
        assert_eq!(Some("abcdefgh"), test.text_content().as_deref());

        // Append end
        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"].into_test_data();
        test.update(data.clone());
        assert_eq!(
            data,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );
        assert_eq!(Some("abcdefghijk"), test.text_content().as_deref());
    }
}

#[cfg(test)]
pub mod test_list_with_iter_map {
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::{
        Element, Text,
        test_helper::{self, TestComp, TestInterface},
    };

    use super::List;

    struct TestData(Vec<&'static str>);
    type TestState = TestComp<TestData>;

    pub struct TestDataViewState {
        list: List<TestItemViewState>,
    }

    pub struct TestItemViewState {
        element: Element,
        text: Text,
    }

    const TEMPLATE_STRING: &str = "<span>?</span>";

    impl TestInterface for TestData {
        type ViewState = TestDataViewState;

        fn init(&self, root: &Element, context: &crate::Context<TestState>) -> Self::ViewState {
            let list = List::new(&root.ws_element(), None, TEMPLATE_STRING);
            let mut view_state = TestDataViewState { list };
            self.update(&mut view_state, context);
            view_state
        }

        fn update(&self, view_state: &mut Self::ViewState, context: &crate::Context<TestState>) {
            view_state.list.update(
                self.0
                    .iter()
                    .enumerate()
                    .map(|(index, value)| (index, value.to_string())),
                &context,
                |template, _item_data, _context| {
                    let element = template.create_element(0);
                    let text = element.ws_node_ref().first_text();
                    TestItemViewState { element, text }
                },
                |view_state, item_data, _context| {
                    view_state.text.update(&item_data.1);
                },
                |vs: &TestItemViewState| &vs.element,
            );
        }
    }

    #[wasm_bindgen_test]
    fn list_with_iter_map_compiles() {
        let test = test_helper::Test::set_up(TestData(Vec::new()));
        assert_eq!(Some(""), test.text_content().as_deref());
        let empty: Vec<&'static str> = Vec::new();
        test.update(TestData(empty.clone()));
    }
}
