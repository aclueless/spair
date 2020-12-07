use crate::utils::PeekableDoubleEnded;
use wasm_bindgen::{JsCast, UnwrapThrowExt};

mod keyed_list_with_render;
pub use keyed_list_with_render::*;

pub trait Keyed<'a> {
    type Key: 'a + Into<Key> + PartialEq<Key>;
    fn key(&self) -> Self::Key;
}

trait UpdateItem<C: crate::component::Component>: crate::dom::ListItem<C> {
    fn update_existing_item(
        &self,
        comp: &crate::component::Comp<C>,
        state: &C,
        old_item: Option<(usize, &mut std::option::Option<(Key, super::Element)>)>,
        new_item: Option<&mut std::option::Option<(Key, super::Element)>>,
        next_sibling: Option<&web_sys::Element>,
        fn_insert: impl FnOnce(&super::Element, Option<&web_sys::Element>),
    ) {
        let mut old_item = old_item.unwrap_throw().1.take();
        fn_insert(&old_item.as_ref().unwrap_throw().1, next_sibling);

        let u = super::ElementUpdater::new(
            comp,
            state,
            &mut old_item.as_mut().unwrap_throw().1,
            super::ElementStatus::Existing,
        );
        self.render(u.into());
        *new_item.expect_throw("Why overflow on new list? - render_item?") = old_item;
    }
}

impl<C: crate::component::Component, T: crate::dom::ListItem<C>> UpdateItem<C> for T {}

#[derive(Default)]
pub struct KeyedList {
    active: Vec<Option<(Key, super::Element)>>,
    // The primary reason for the double buffer is for easy implementation, performance go after.
    buffer: Vec<Option<(Key, super::Element)>>,
    template: Option<super::Element>,
    old_elements_map: std::collections::HashMap<Key, OldElement>,
}

impl Clone for KeyedList {
    fn clone(&self) -> Self {
        // No clone for keyed list
        Self {
            active: Vec::new(),
            buffer: Vec::new(),
            old_elements_map: std::collections::HashMap::new(),
            template: None,
        }
    }
}

impl KeyedList {
    pub fn create_context<'a>(
        &'a mut self,
        root_item_tag: &str,
        new_item_count: usize,
        parent: &'a web_sys::Node,
        use_template: bool,
    ) -> KeyedListContext<'a> {
        self.pre_update(new_item_count);

        let require_init_template = use_template && self.template.is_none();
        if require_init_template {
            self.template = Some(super::Element::new_ns(None, root_item_tag));
        }
        let template = self.template.as_mut();
        let new_item_count = self.active.len();
        KeyedListContext {
            parent,
            old: self.buffer.iter_mut().enumerate().peekable_double_ended(),
            new: self.active.iter_mut().peekable_double_ended(),
            old_elements_map: &mut self.old_elements_map,
            new_item_count,
            next_sibling: None,
            template,
            require_init_template,
        }
    }

    // TODO better name?
    fn pre_update(&mut self, count: usize) {
        self.old_elements_map.reserve(count);
        if count < self.buffer.len() {
            self.buffer.truncate(count);
        } else {
            self.buffer
                .extend((0..(count - self.buffer.len())).map(|_| None));
        }
        debug_assert_eq!(count, self.buffer.len());
        std::mem::swap(&mut self.active, &mut self.buffer);
    }

    pub fn clear(&mut self, parent: &web_sys::Node) {
        self.active.iter().for_each(|item| {
            item.as_ref()
                .expect_throw("Item should exist - clear")
                .1
                .remove_from(parent)
        });
    }

    pub fn append_to(&self, parent: &web_sys::Node) {
        self.active.iter().for_each(|item| {
            item.as_ref()
                .expect_throw("Item should exist - append_to")
                .1
                .append_to(parent)
        });
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum Key {
    String(String),
    Signed(i64),
    Unsigned(u64),
}

impl From<&str> for Key {
    fn from(value: &str) -> Self {
        Key::String(value.to_string())
    }
}

impl From<i64> for Key {
    fn from(value: i64) -> Self {
        Key::Signed(value)
    }
}

impl From<u64> for Key {
    fn from(value: u64) -> Self {
        Key::Unsigned(value)
    }
}

impl PartialEq<Key> for &str {
    fn eq(&self, other: &Key) -> bool {
        match other {
            Key::String(value) => value == self,
            _ => false,
        }
    }
}

impl PartialEq<Key> for i64 {
    fn eq(&self, other: &Key) -> bool {
        match other {
            Key::Signed(value) => value == self,
            _ => false,
        }
    }
}

impl PartialEq<Key> for u64 {
    fn eq(&self, other: &Key) -> bool {
        match other {
            Key::Unsigned(value) => value == self,
            _ => false,
        }
    }
}

pub struct KeyedListContext<'a> {
    parent: &'a web_sys::Node,
    old: crate::utils::PeekableDoubleEndedIterator<
        std::iter::Enumerate<std::slice::IterMut<'a, Option<(Key, super::Element)>>>,
    >,
    new: crate::utils::PeekableDoubleEndedIterator<
        std::slice::IterMut<'a, Option<(Key, super::Element)>>,
    >,
    old_elements_map: &'a mut std::collections::HashMap<Key, OldElement>,
    new_item_count: usize,
    next_sibling: Option<web_sys::Element>,
    template: Option<&'a mut super::Element>,
    require_init_template: bool,
}

pub struct KeyedListUpdater<'a, C> {
    pub comp: &'a crate::component::Comp<C>,
    pub state: &'a C,

    pub(super) list_context: KeyedListContext<'a>,
}

impl<'a, C: crate::component::Component> KeyedListUpdater<'a, C> {
    fn create_element_for_new_item(&self, tag: &str) -> (super::Element, super::ElementStatus) {
        match &self.list_context.template {
            Some(template) => (Clone::clone(*template), super::ElementStatus::JustCloned),
            None => (
                super::Element::new_ns(None, tag),
                super::ElementStatus::JustCreated,
            ),
        }
    }

    pub fn update<I>(
        &mut self,
        items_state_iter: impl Iterator<Item = I> + DoubleEndedIterator,
    ) -> super::RememberSettingSelectedOption
    where
        for<'k> I: Keyed<'k> + super::ListItem<C>,
    {
        // No items? Just clear the current list.
        if self.list_context.new_item_count == 0 {
            self.remove_all_old_items();
            return super::RememberSettingSelectedOption;
        }

        let mut items_state_iter = items_state_iter.peekable_double_ended();
        if self.list_context.require_init_template {
            let u = super::ElementUpdater::new(
                self.comp,
                self.state,
                self.list_context.template.as_mut().unwrap(),
                super::ElementStatus::JustCreated,
            );

            items_state_iter.peek().unwrap_throw().render(u.into());
        }
        loop {
            let mut count = self.update_same_key_items_from_start(&mut items_state_iter);
            count += self.update_same_key_items_from_end(&mut items_state_iter);
            count += self.update_moved_forward_item(&mut items_state_iter);
            count += self.update_moved_backward_item(&mut items_state_iter);
            if count == 0 {
                break;
            }
        }

        self.update_other_items_in_middle(&mut items_state_iter);
        super::RememberSettingSelectedOption
    }

    fn update_same_key_items_from_start<I>(
        &mut self,
        items_state_iter: &mut crate::utils::PeekableDoubleEndedIterator<impl Iterator<Item = I>>,
    ) -> usize
    where
        for<'k> I: Keyed<'k> + super::ListItem<C>,
    {
        let mut count = 0;
        loop {
            match (items_state_iter.peek(), self.list_context.old.peek()) {
                (Some(item_state), Some(item)) => {
                    let item = item
                        .1
                        .as_ref()
                        .expect_throw("Why an old item None? - update_same_key_items_from_start");
                    if !item_state.key().eq(&item.0) {
                        return count;
                    }
                }
                _ => return count,
            }
            count += 1;
            items_state_iter.next().unwrap_throw().update_existing_item(
                self.comp,
                self.state,
                self.list_context.old.next(),
                self.list_context.new.next(),
                None,
                |_, _| {},
            );
        }
    }

    fn update_same_key_items_from_end<I>(
        &mut self,
        items_state_iter: &mut crate::utils::PeekableDoubleEndedIterator<
            impl Iterator<Item = I> + DoubleEndedIterator,
        >,
    ) -> usize
    where
        for<'k> I: Keyed<'k> + super::ListItem<C>,
    {
        let mut count = 0;
        loop {
            let ws_element = match (
                items_state_iter.peek_back(),
                self.list_context.old.peek_back(),
            ) {
                (Some(item_state), Some(item)) => {
                    let item = item
                        .1
                        .as_ref()
                        .expect_throw("Why an old item None? - update_same_key_items_from_end");

                    if !item_state.key().eq(&item.0) {
                        return count;
                    }
                    item.1.ws_element().clone()
                }
                _ => return count,
            };
            count += 1;
            items_state_iter
                .next_back()
                .unwrap_throw()
                .update_existing_item(
                    self.comp,
                    self.state,
                    self.list_context.old.next_back(),
                    self.list_context.new.next_back(),
                    None,
                    |_, _| {},
                );
            self.list_context.next_sibling = Some(ws_element);
        }
    }

    fn update_moved_forward_item<I>(
        &mut self,
        items_state_iter: &mut crate::utils::PeekableDoubleEndedIterator<impl Iterator<Item = I>>,
    ) -> usize
    where
        for<'k> I: Keyed<'k> + super::ListItem<C>,
    {
        match (items_state_iter.peek(), self.list_context.old.peek_back()) {
            (Some(item_state), Some(item)) => {
                let item = item
                    .1
                    .as_ref()
                    .expect_throw("Why an old item None? - update_same_key_items_from_end");
                if !item_state.key().eq(&item.0) {
                    return 0;
                }
            }
            _ => return 0,
        }
        let moved = self.list_context.old.next_back();
        let next_sibling = self
            .list_context
            .old
            .peek()
            .and_then(|item| item.1.as_ref().map(|item| item.1.ws_element()));
        let parent = self.list_context.parent;
        items_state_iter.next().unwrap_throw().update_existing_item(
            self.comp,
            self.state,
            moved,
            self.list_context.new.next(),
            next_sibling,
            |element, next_sibling| {
                element.insert_before(parent, next_sibling.map(|element| element.unchecked_ref()));
            },
        );
        1
    }

    fn update_moved_backward_item<I>(
        &mut self,
        items_state_iter: &mut crate::utils::PeekableDoubleEndedIterator<
            impl Iterator<Item = I> + DoubleEndedIterator,
        >,
    ) -> usize
    where
        for<'k> I: Keyed<'k> + super::ListItem<C>,
    {
        let new_next_sibling = match (items_state_iter.peek_back(), self.list_context.old.peek()) {
            (Some(item_state), Some(item)) => {
                let item = item
                    .1
                    .as_ref()
                    .expect_throw("Why an old item None? - update_same_key_items_from_end");
                if !item_state.key().eq(&item.0) {
                    return 0;
                }
                item.1.ws_element().clone()
            }
            _ => return 0,
        };
        items_state_iter
            .next_back()
            .unwrap_throw()
            .update_existing_item(
                self.comp,
                self.state,
                self.list_context.old.next(),
                self.list_context.new.next_back(),
                self.list_context.next_sibling.as_ref(),
                |element, next_sibling| {
                    element.insert_before(
                        self.list_context.parent,
                        next_sibling.map(|element| element.unchecked_ref()),
                    );
                },
            );
        self.list_context.next_sibling = Some(new_next_sibling);
        1
    }

    fn update_other_items_in_middle<I>(
        &mut self,
        items_state_iter: &mut crate::utils::PeekableDoubleEndedIterator<impl Iterator<Item = I>>,
    ) where
        for<'k> I: Keyed<'k> + super::ListItem<C>,
    {
        if items_state_iter.peek().is_none() {
            self.remove_remain_items();
            return;
        }

        if self.list_context.old.peek().is_none() {
            self.insert_remain_items(items_state_iter);
            return;
        }

        self.construct_old_elements_map_from_remaining_old_elements();

        // Using longest_increasing_subsequence find which elements should be moved around in the browser's DOM
        // and which should be stay still
        let mut items_with_lis: Vec<_> = items_state_iter
            .map(|item| {
                let old_element = self
                    .list_context
                    .old_elements_map
                    .remove(&item.key().into());
                ItemWithLis::new(item, old_element)
            })
            .collect();
        longest_increasing_subsequence(&mut items_with_lis);

        self.remove_old_elements_that_still_in_old_elements_map();

        for ItemWithLis {
            item_state,
            old_element,
            lis,
        } in items_with_lis.into_iter().rev()
        {
            let (mut element, status) = match old_element {
                Some(old_element) => (old_element.element, super::ElementStatus::Existing),
                None => self.create_element_for_new_item(I::ROOT_ELEMENT_TAG),
            };

            let u = super::ElementUpdater::new(self.comp, self.state, &mut element, status);

            item_state.render(u.into());
            if !lis {
                let next_sibling = self
                    .list_context
                    .next_sibling
                    .as_ref()
                    .map(|element| element.unchecked_ref());
                element.insert_before(self.list_context.parent, next_sibling);
            }

            self.list_context.next_sibling = Some(element.ws_element().clone());
            *self
                .list_context
                .new
                .next_back()
                .expect_throw("Why new-list overflow?") = Some((item_state.key().into(), element));
        }
    }

    fn construct_old_elements_map_from_remaining_old_elements(&mut self) {
        self.list_context.old_elements_map.clear();
        while let Some((index, item)) = self.list_context.old.next() {
            let (key, element) = item.take().expect_throw(
                "Why no item in old list? - construct_old_elements_map_from_remaining_old_elements",
            );
            self.list_context
                .old_elements_map
                .insert(key, OldElement { index, element });
        }
    }

    fn remove_old_elements_that_still_in_old_elements_map(&mut self) {
        let parent = self.list_context.parent;
        self.list_context
            .old_elements_map
            .drain()
            .for_each(|(_, item)| {
                item.element.remove_from(parent);
            })
    }

    fn remove_all_old_items(&mut self) {
        self.list_context.parent.set_text_content(None);
        while let Some((_, item)) = self.list_context.old.next() {
            item.take()
                .expect_throw("Why no item in old list? - remove_all_old_items");
        }
    }

    fn remove_remain_items(&mut self) {
        let parent = self.list_context.parent;
        while let Some((_, item)) = self.list_context.old.next() {
            item.take()
                .expect_throw("Why no item in old list? - remove_remain_items")
                .1
                .remove_from(parent);
        }
    }

    fn insert_remain_items<I>(
        &mut self,
        items_state_iter: &mut crate::utils::PeekableDoubleEndedIterator<impl Iterator<Item = I>>,
    ) where
        for<'k> I: Keyed<'k> + super::ListItem<C>,
    {
        for item_state in items_state_iter {
            let (mut element, status) = self.create_element_for_new_item(I::ROOT_ELEMENT_TAG);

            let u = super::ElementUpdater::new(self.comp, self.state, &mut element, status);

            item_state.render(u.into());
            element.insert_before(
                self.list_context.parent,
                self.list_context
                    .next_sibling
                    .as_ref()
                    .map(|element| element.unchecked_ref()),
            );
            *self
                .list_context
                .new
                .next()
                .expect_throw("new remain items") = Some((item_state.key().into(), element));
        }
    }
}

#[derive(Debug)]
struct ItemWithLis<I> {
    item_state: I,
    old_element: Option<OldElement>,
    lis: bool,
}

#[derive(Debug)]
struct OldElement {
    index: usize,
    element: super::Element,
}

impl<I> ItemWithLis<I> {
    fn new(item_state: I, old_element: Option<OldElement>) -> Self {
        Self {
            item_state,
            old_element,
            lis: false,
        }
    }
}

// Copied from https://github.com/axelf4/lis and modified to work with Spair.
fn longest_increasing_subsequence<I>(items: &mut [ItemWithLis<I>]) {
    let mut p = vec![0; items.len()];
    // indices of the new items
    let mut m = Vec::with_capacity(items.len());
    // only iter through items with old index
    let mut it = items
        .iter()
        .enumerate()
        .filter(|(_, x)| x.old_element.is_some());
    if let Some((i, _)) = it.next() {
        m.push(i);
    } else {
        return;
    }

    for (i, x) in it {
        // Test whether a[i] can extend the current sequence
        if items[*m.last().unwrap_throw()]
            .old_element
            .as_ref()
            .unwrap_throw()
            .index
            .cmp(&x.old_element.as_ref().unwrap_throw().index)
            == std::cmp::Ordering::Less
        {
            p[i] = *m.last().unwrap_throw();
            m.push(i);
            continue;
        }

        // Binary search for largest j ≤ m.len() such that a[m[j]] < a[i]
        let j = match m.binary_search_by(|&j| {
            items[j]
                .old_element
                .as_ref()
                .unwrap_throw()
                .index
                .cmp(&x.old_element.as_ref().unwrap_throw().index)
                .then(std::cmp::Ordering::Greater)
        }) {
            Ok(j) | Err(j) => j,
        };
        if j > 0 {
            p[i] = m[j - 1];
        }
        m[j] = i;
    }

    // Reconstruct the longest increasing subsequence
    let mut k = *m.last().unwrap_throw();
    for _ in (0..m.len()).rev() {
        items[k].lis = true;
        k = p[k];
    }
}

#[cfg(test)]
mod keyed_list_tests {
    use wasm_bindgen::UnwrapThrowExt;
    use wasm_bindgen_test::*;
    impl super::ItemWithLis<&()> {
        fn index(index: usize) -> Self {
            Self {
                item_state: &(),
                old_element: Some(super::OldElement {
                    index,
                    element: super::super::Element::new_ns(None, "div"),
                }),
                lis: false,
            }
        }
        fn none() -> Self {
            Self {
                item_state: &(),
                old_element: None,
                lis: false,
            }
        }
    }

    fn collect_lis(mut items: Vec<super::ItemWithLis<&()>>) -> Vec<usize> {
        super::longest_increasing_subsequence(&mut items[..]);
        items
            .iter()
            .flat_map(|item| {
                if item.lis {
                    item.old_element
                        .as_ref()
                        .map(|old_element| old_element.index)
                } else {
                    None
                }
            })
            .collect()
    }

    fn collect_lis_from_index(indices: &[usize]) -> Vec<usize> {
        let items = indices
            .iter()
            .map(|i| super::ItemWithLis::index(*i))
            .collect();
        collect_lis(items)
    }

    #[wasm_bindgen_test]
    fn lis_with_none() {
        let items = vec![
            super::ItemWithLis::index(5),
            super::ItemWithLis::index(1),
            super::ItemWithLis::index(3),
            super::ItemWithLis::none(),
            super::ItemWithLis::index(6),
            super::ItemWithLis::index(8),
            super::ItemWithLis::none(),
            super::ItemWithLis::index(9),
            super::ItemWithLis::index(0),
            super::ItemWithLis::index(7),
        ];
        let rs = collect_lis(items);
        let expected = [1, 3, 6, 8, 9];
        assert_eq!(&expected[..], &rs[..]);
    }

    #[wasm_bindgen_test]
    fn lis() {
        // Why this produces different result than https://github.com/axelf4/lis?
        // But it produces the same result like https://en.wikipedia.org/wiki/Longest_increasing_subsequence?
        let rs = collect_lis_from_index(&[0, 8, 4, 12, 2, 10, 6, 14, 1, 9, 5, 13, 3, 11, 7, 15]);
        assert_eq!(rs, [0, 2, 6, 9, 11, 15]);

        assert!(collect_lis_from_index(&[]).is_empty());

        let rs = collect_lis_from_index(&[5, 1, 3, 6, 8, 9, 0, 7, 10, 5, 2]);
        assert_eq!(rs, [1, 3, 6, 8, 9, 10]);

        let rs = collect_lis_from_index(&[5, 7, 2, 5, 0, 3, 8, 4, 1, 6, 5, 9]);
        assert_eq!(rs, [0, 3, 4, 5, 9]);
    }

    impl crate::component::Component for () {
        type Routes = ();
        fn render(&self, _: crate::Element<Self>) {}
    }

    struct PhantomApp {
        root: super::super::Element,
        _rc: crate::component::RcComp<()>,
        comp: crate::component::Comp<()>,
    }

    impl PhantomApp {
        fn new() -> Self {
            let root = super::super::Element::new_ns(None, "div");
            let _rc = crate::component::RcComp::new(Some(root.ws_element().clone()));
            _rc.set_state(());

            let comp = _rc.comp();
            Self { root, _rc, comp }
        }

        fn updater(&mut self) -> super::super::HtmlUpdater<()> {
            super::super::ElementUpdater::new(
                &self.comp,
                &(),
                &mut self.root,
                super::super::ElementStatus::Existing,
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

    impl crate::dom::ListItem<()> for &&'static str {
        const ROOT_ELEMENT_TAG: &'static str = "span";
        fn render(&self, span: crate::Element<()>) {
            span.render(**self);
        }
    }

    impl<'a> super::Keyed<'a> for &&'static str {
        type Key = &'a str;
        fn key(&self) -> Self::Key {
            self
        }
    }

    #[wasm_bindgen_test]
    fn keyed_list_with_template() {
        keyed_list(super::super::ListElementCreation::Clone);
    }

    #[wasm_bindgen_test]
    fn keyed_list_no_template() {
        keyed_list(super::super::ListElementCreation::New);
    }

    fn keyed_list(mode: super::super::ListElementCreation) {
        // [a, b, c, d, e, f, g, h, i, j, k]
        // [f, b, d, l, g, i, m, j, a, h, k]

        wasm_logger::init(wasm_logger::Config::default());

        let mut pa = PhantomApp::new();

        let empty: Vec<&'static str> = Vec::new();
        let _ = pa.updater().keyed_list(&empty, mode);
        assert_eq!(Some(""), pa.root.ws_element().text_content().as_deref());
        assert_eq!(empty, pa.collect_from_keyed_list());

        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"];
        let _ = pa.updater().keyed_list(&data, mode);
        assert_eq!(data, pa.collect_from_keyed_list());
        assert_eq!(
            Some("abcdefghijk"),
            pa.root.ws_element().text_content().as_deref()
        );

        // Random shuffle + addition
        let data = vec!["f", "b", "d", "l", "g", "i", "m", "j", "a", "h", "k"];
        let _ = pa.updater().keyed_list(&data, mode);
        assert_eq!(
            Some("fbdlgimjahk"),
            pa.root.ws_element().text_content().as_deref()
        );
        assert_eq!(data, pa.collect_from_keyed_list());

        // Empty the list
        let _ = pa.updater().keyed_list(&empty, mode);
        assert_eq!(Some(""), pa.root.ws_element().text_content().as_deref());
        assert_eq!(empty, pa.collect_from_keyed_list());

        // Add back
        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"];
        let _ = pa.updater().keyed_list(&data, mode);
        assert_eq!(data, pa.collect_from_keyed_list());
        assert_eq!(
            Some("abcdefghijk"),
            pa.root.ws_element().text_content().as_deref()
        );

        // Forward
        let data = vec!["a", "i", "b", "c", "d", "e", "f", "g", "h", "j", "k"];
        let _ = pa.updater().keyed_list(&data, mode);
        assert_eq!(data, pa.collect_from_keyed_list());
        assert_eq!(
            Some("aibcdefghjk"),
            pa.root.ws_element().text_content().as_deref()
        );

        // Backward
        let data = vec!["a", "i", "c", "d", "e", "f", "g", "h", "b", "j", "k"];
        let _ = pa.updater().keyed_list(&data, mode);
        assert_eq!(data, pa.collect_from_keyed_list());
        assert_eq!(
            Some("aicdefghbjk"),
            pa.root.ws_element().text_content().as_deref()
        );

        // Swap
        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"];
        let _ = pa.updater().keyed_list(&data, mode);
        assert_eq!(data, pa.collect_from_keyed_list());
        assert_eq!(
            Some("abcdefghijk"),
            pa.root.ws_element().text_content().as_deref()
        );

        // Remove middle
        let data = vec!["a", "b", "c", "d", "i", "j", "k"];
        let _ = pa.updater().keyed_list(&data, mode);
        assert_eq!(data, pa.collect_from_keyed_list());
        assert_eq!(
            Some("abcdijk"),
            pa.root.ws_element().text_content().as_deref()
        );

        // Insert middle
        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"];
        let _ = pa.updater().keyed_list(&data, mode);
        assert_eq!(data, pa.collect_from_keyed_list());
        assert_eq!(
            Some("abcdefghijk"),
            pa.root.ws_element().text_content().as_deref()
        );

        // Remove start
        let data = vec!["d", "e", "f", "g", "h", "i", "j", "k"];
        let _ = pa.updater().keyed_list(&data, mode);
        assert_eq!(data, pa.collect_from_keyed_list());
        assert_eq!(
            Some("defghijk"),
            pa.root.ws_element().text_content().as_deref()
        );

        // Insert start
        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"];
        let _ = pa.updater().keyed_list(&data, mode);
        assert_eq!(data, pa.collect_from_keyed_list());
        assert_eq!(
            Some("abcdefghijk"),
            pa.root.ws_element().text_content().as_deref()
        );

        // Remove end
        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h"];
        let _ = pa.updater().keyed_list(&data, mode);
        assert_eq!(data, pa.collect_from_keyed_list());
        assert_eq!(
            Some("abcdefgh"),
            pa.root.ws_element().text_content().as_deref()
        );

        // Append end
        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"];
        let _ = pa.updater().keyed_list(&data, mode);
        assert_eq!(data, pa.collect_from_keyed_list());
        assert_eq!(
            Some("abcdefghijk"),
            pa.root.ws_element().text_content().as_deref()
        );
    }
}
