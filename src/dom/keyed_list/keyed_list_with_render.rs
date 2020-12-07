use crate::utils::PeekableDoubleEnded;
use wasm_bindgen::{JsCast, UnwrapThrowExt};

// pub trait FromElementUpdater<'a, C> {
//     fn from(u: crate::dom::ElementUpdater<'a, C>) -> Self;
// }

// impl<'a, C> FromElementUpdater<'a, C> for crate::dom::HtmlUpdater<'a, C> {
//     fn from(u: crate::dom::ElementUpdater<'a, C>) -> Self {
//         From::from(u)
//     }
// }

impl<'a, C: crate::component::Component> super::KeyedListUpdater<'a, C> {
    pub fn update_with_render<I, G, K, R, U>(
        &mut self,
        items_state_iter: impl Iterator<Item = I> + DoubleEndedIterator,
        get_key: G,
        render: R,
    ) -> crate::dom::RememberSettingSelectedOption
    where
        for<'i> G: Fn(&'i I) -> K,
        K: 'a + Into<super::Key> + PartialEq<super::Key>,
        for<'i> R: Fn(&'i I, U),
        for<'c> U: From<crate::dom::ElementUpdater<'c, C>>,
    {
        // No items? Just clear the current list.
        if self.list_context.new_item_count == 0 {
            self.remove_all_old_items();
            return crate::dom::RememberSettingSelectedOption;
        }

        let mut items_state_iter = items_state_iter.peekable_double_ended();
        if self.list_context.require_init_template {
            let u = crate::dom::ElementUpdater::new(
                self.comp,
                self.state,
                self.list_context.template.as_mut().unwrap(),
                crate::dom::ElementStatus::JustCreated,
            );
            render(items_state_iter.peek().unwrap_throw(), u.into());
        }
        // loop {
        //     let mut count = self.update_same_key_items_from_start(&mut items_state_iter);
        //     count += self.update_same_key_items_from_end(&mut items_state_iter);
        //     count += self.update_moved_forward_item(&mut items_state_iter);
        //     count += self.update_moved_backward_item(&mut items_state_iter);
        //     if count == 0 {
        //         break;
        //     }
        // }

        // self.update_other_items_in_middle(&mut items_state_iter);

        crate::dom::RememberSettingSelectedOption
    }
}

#[cfg(test)]
mod keyed_list_with_render_tests {
    use wasm_bindgen::UnwrapThrowExt;
    use wasm_bindgen_test::*;
    struct Unit;
    impl crate::component::Component for Unit {
        type Routes = ();
        fn render(&self, _: crate::Element<Self>) {}
    }

    struct PhantomApp {
        root: crate::dom::Element,
        _rc: crate::component::RcComp<Unit>,
        comp: crate::component::Comp<Unit>,
    }

    impl PhantomApp {
        fn new() -> Self {
            let root = crate::dom::Element::new_ns(None, "div");
            let _rc = crate::component::RcComp::new(Some(root.ws_element().clone()));
            _rc.set_state(Unit);

            let comp = _rc.comp();
            Self { root, _rc, comp }
        }

        fn updater(&mut self) -> crate::dom::HtmlUpdater<Unit> {
            crate::dom::ElementUpdater::new(
                &self.comp,
                &Unit,
                &mut self.root,
                crate::dom::ElementStatus::Existing,
            )
            .into()
        }

        fn collect_from_keyed_list(&self) -> Vec<String> {
            if let crate::dom::nodes::Node::KeyedList(kl) = self.root.nodes.0.first().unwrap_throw()
            {
                kl.active
                    .iter()
                    .map(|item| {
                        item.as_ref()
                            .unwrap_throw()
                            .1
                            .nodes
                            .0
                            .first()
                            .unwrap_throw()
                    })
                    .map(|item| match item {
                        crate::dom::nodes::Node::Text(text) => text.text.clone(),
                        _ => panic!("Should be a text?"),
                    })
                    .collect()
            } else {
                Vec::new()
            }
        }
    }

    fn render(item: &&str, span: crate::Element<()>) {
        span.render(*item);
    }

    fn get_key<'a>(item: &'a &str) -> &'a str {
        *item
    }

    #[wasm_bindgen_test]
    fn keyed_list_with_template() {
        keyed_list(crate::dom::ListElementCreation::Clone);
    }

    #[wasm_bindgen_test]
    fn keyed_list_no_template() {
        keyed_list(crate::dom::ListElementCreation::New);
    }

    fn keyed_list(mode: crate::dom::ListElementCreation) {
        let mut pa = PhantomApp::new();

        let empty: Vec<&'static str> = Vec::new();
        let _ = pa
            .updater()
            .keyed_list_with_render(&empty, mode, "span", get_key, render);
        assert_eq!(Some(""), pa.root.ws_element().text_content().as_deref());
        assert_eq!(empty, pa.collect_from_keyed_list());
    }
}
