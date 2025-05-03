use std::{hash::Hash, iter::Enumerate, slice::IterMut};

use rustc_hash::FxHashMap as HashMap;
use wasm_bindgen::UnwrapThrowExt;

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

pub struct KeyedList<C, I>
where
    I: KeyedListItemView<C>,
    C: Component,
{
    parent_element: WsElement,
    template: TemplateElement,
    end_node_marker_for_partial_list: Option<web_sys::Node>,
    active_items: Vec<Option<I::ViewState>>,
    buffer_items: Vec<Option<I::ViewState>>,
    active_items_map: HashMap<I::Key, OldItem<I::ViewState>>,
}

struct KeyedListUpdater<'a, C, I>
where
    I: KeyedListItemView<C>,
    C: Component,
{
    parent_element: &'a WsElement,
    template: &'a TemplateElement,
    old_list: PeekableDoubleEndedIterator<Enumerate<IterMut<'a, Option<I::ViewState>>>>,
    new_list: PeekableDoubleEndedIterator<IterMut<'a, Option<I::ViewState>>>,
    old_items_map: &'a mut HashMap<I::Key, OldItem<I::ViewState>>,
    end_flag_for_the_next_rendered_item_bottom_up: Option<web_sys::Node>,
}

pub struct OldItem<VS> {
    pub index: usize,
    pub view_state: VS,
}

impl<C, I> KeyedList<C, I>
where
    I: KeyedListItemView<C> + 'static,
    C: Component + 'static,
{
    pub fn new(
        parent_element: &WsElement,
        end_node_marker_for_partial_list: Option<web_sys::Node>,
    ) -> Self {
        Self {
            parent_element: parent_element.clone(),
            template: TemplateElement::new(I::template_string()),
            end_node_marker_for_partial_list,
            active_items: Vec::new(),
            buffer_items: Vec::new(),
            active_items_map: HashMap::default(),
        }
    }

    pub fn update<'a>(&mut self, item_data: impl Iterator<Item = &'a I>, context: &Context<C>) {
        // Current implementation requires knowing the exact number in advance.
        let item_data: Vec<_> = item_data.collect();
        let new_count = item_data.len();

        self.active_items_map.reserve(new_count);
        if new_count < self.buffer_items.len() {
            self.buffer_items.truncate(new_count);
        } else {
            self.buffer_items
                .extend((0..(new_count - self.buffer_items.len())).map(|_| None));
        }
        std::mem::swap(&mut self.active_items, &mut self.buffer_items);

        if new_count == 0 {
            self.remove_all_old_items();
            return;
        }

        let mut keyed_list_updater = KeyedListUpdater {
            parent_element: &self.parent_element,
            template: &self.template,
            old_list: self
                .buffer_items
                .iter_mut()
                .enumerate()
                .peekable_double_ended(),
            new_list: self.active_items.iter_mut().peekable_double_ended(),
            old_items_map: &mut self.active_items_map,
            end_flag_for_the_next_rendered_item_bottom_up: self
                .end_node_marker_for_partial_list
                .clone(),
        };
        keyed_list_updater.update(item_data, context);
    }

    fn remove_all_old_items(&mut self) {
        if self.end_node_marker_for_partial_list.is_none() {
            self.parent_element.clear_text_content();
            for item in self.active_items.iter_mut() {
                item.take();
            }
        } else {
            for item in self.active_items.iter_mut() {
                if let Some(item) = item.take() {
                    self.parent_element.remove_child(I::root_element(&item));
                };
            }
        }
    }
}

impl<C, I> KeyedListUpdater<'_, C, I>
where
    I: KeyedListItemView<C> + 'static,
    C: Component + 'static,
{
    fn update(&mut self, item_data: Vec<&I>, context: &Context<C>) {
        let mut item_data = item_data.into_iter().peekable_double_ended();
        loop {
            let mut count = self.update_same_items_from_start(&mut item_data, context);
            count += self.update_same_items_from_end(&mut item_data, context);
            count += self.update_moved_forward_item(&mut item_data, context);
            count += self.update_moved_backward_item(&mut item_data, context);
            if count == 0 {
                break;
            }
        }
        self.update_items_in_the_middle(&mut item_data, context);
    }

    fn update_same_items_from_start(
        &mut self,
        item_data: &mut PeekableDoubleEndedIterator<std::vec::IntoIter<&I>>,
        context: &Context<C>,
    ) -> usize {
        let mut count = 0;
        loop {
            // Check if the first items from `items` and `old_list` have the same key.
            if let (Some(item_data), Some(old_view_state)) =
                (item_data.peek(), self.old_list.peek())
            {
                let old_view_state = old_view_state
                    .1
                    .as_ref()
                    .expect_throw("keyed_list::KeyedListUpdater::update_same_items_from_start");
                if !I::key_from_view_state(old_view_state).eq(item_data.get_key()) {
                    return count;
                }
            } else {
                return count;
            }

            // Yes, they have the same key, then update the old item
            count += 1;
            Self::update_existing_item(
                self.parent_element,
                item_data.next().unwrap_throw(),
                self.old_list.next(),
                self.new_list.next(),
                None,
                // self.old_list.peek().and_then(|next_old_item| {
                //     next_old_item
                //         .1
                //         .as_ref()
                //         .map(|updater| updater.root_element())
                // }),
                false,
                context,
            );
        }
    }

    fn update_existing_item(
        parent_element: &WsElement,
        item_data: &I,
        old_view_state: Option<(usize, &mut Option<I::ViewState>)>,
        new_view_state: Option<&mut Option<I::ViewState>>,
        next_sibling: Option<&web_sys::Node>,
        relocating_item: bool,
        context: &Context<C>,
    ) {
        let mut old_view_state = old_view_state.unwrap_throw().1.take().unwrap_throw();
        if relocating_item {
            parent_element
                .insert_new_node_before_a_node(I::root_element(&old_view_state), next_sibling);
        }
        I::update(&mut old_view_state, item_data, context);
        if let Some(new_view_state) = new_view_state {
            *new_view_state = Some(old_view_state);
        }
    }

    fn update_same_items_from_end(
        &mut self,
        item_data: &mut PeekableDoubleEndedIterator<std::vec::IntoIter<&I>>,
        context: &Context<C>,
    ) -> usize {
        let mut count = 0;
        loop {
            let new_end_flag = if let (Some(item_data), Some(old_view_state)) =
                (item_data.peek_back(), self.old_list.peek_back())
            {
                let old_view_state = old_view_state
                    .1
                    .as_ref()
                    .expect_throw("keyed_list::KeyedListUpdater::update_same_items_from_end");

                if !I::key_from_view_state(old_view_state).eq(item_data.get_key()) {
                    return count;
                }
                // The keys are the same, so we need to continue the loop.
                // The proccessed item  will be upped by one from the bottom.
                // And we will need to update `end_flag_for_the_next_entry_bottom_up`
                // with this value
                I::root_element(old_view_state).web_sys_node().clone()
            } else {
                return count;
            };
            count += 1;
            Self::update_existing_item(
                self.parent_element,
                item_data.next_back().unwrap_throw(),
                self.old_list.next_back(),
                self.new_list.next_back(),
                None,
                // self.end_flag_for_the_next_entry_bottom_up.as_ref(),
                false,
                context,
            );
            self.end_flag_for_the_next_rendered_item_bottom_up = Some(new_end_flag);
        }
    }

    fn update_moved_forward_item(
        &mut self,
        item_data: &mut PeekableDoubleEndedIterator<std::vec::IntoIter<&I>>,
        context: &Context<C>,
    ) -> usize {
        if let (Some(item_data), Some(old_view_state)) =
            (item_data.peek(), self.old_list.peek_back())
        {
            let old_view_state = old_view_state
                .1
                .as_ref()
                .expect_throw("keyed_list::KeyedListUpdater::update_moved_forward_item");
            if !I::key_from_view_state(old_view_state).eq(item_data.get_key()) {
                // No entry moved forward
                return 0;
            }
        } else {
            return 0;
        }

        let moved = self.old_list.next_back();
        let next_sibling = self.old_list.peek().and_then(|item| {
            item.1
                .as_ref()
                .map(|view_state| I::root_element(view_state).web_sys_node())
        });
        Self::update_existing_item(
            self.parent_element,
            item_data.next().unwrap_throw(),
            moved,
            self.new_list.next(),
            next_sibling,
            true,
            context,
        );
        1
    }

    fn update_moved_backward_item(
        &mut self,
        item_data: &mut PeekableDoubleEndedIterator<std::vec::IntoIter<&I>>,
        context: &Context<C>,
    ) -> usize {
        let new_end_flag = if let (Some(item_data), Some(old_view_state)) =
            (item_data.peek_back(), self.old_list.peek())
        {
            let old_view_state = old_view_state
                .1
                .as_ref()
                .expect_throw("keyed_list::KeyedListUpdater::update_moved_backward_item");
            if !I::key_from_view_state(old_view_state).eq(item_data.get_key()) {
                // No entry moved backward
                return 0;
            }
            I::root_element(old_view_state).web_sys_node().clone()
        } else {
            return 0;
        };
        Self::update_existing_item(
            self.parent_element,
            item_data.next_back().unwrap_throw(),
            self.old_list.next(),
            self.new_list.next_back(),
            self.end_flag_for_the_next_rendered_item_bottom_up.as_ref(),
            true,
            context,
        );
        self.end_flag_for_the_next_rendered_item_bottom_up = Some(new_end_flag);
        1
    }

    fn update_items_in_the_middle(
        &mut self,
        item_data: &mut PeekableDoubleEndedIterator<std::vec::IntoIter<&I>>,
        context: &Context<C>,
    ) {
        if item_data.peek().is_none() {
            self.remove_items_still_in_old_list();
            return;
        }

        if self.old_list.peek().is_none() {
            self.insert_new_items_in_the_middle(item_data, context);
            return;
        }

        self.construct_old_items_map_from_remaining_old_items();

        // Using longest_increasing_subsequence to find which elements should be moved around in the browser's DOM
        // and which should be stay still
        let mut item_data_with_lis: Vec<_> = item_data
            .map(|item_data| {
                let key = item_data.get_key();
                let old_view_state = self.old_items_map.remove(key);
                ItemWithLis::new(item_data, old_view_state)
            })
            .collect();
        longest_increasing_subsequence(&mut item_data_with_lis);

        self.remove_old_items_that_still_in_old_items_map();

        for iwl in item_data_with_lis.into_iter().rev() {
            let ItemWithLis {
                item_data,
                old_view_state,
                is_in_lis: lis,
            } = iwl;

            let view_state = match old_view_state {
                Some(old) => {
                    let mut view_state = old.view_state;
                    I::update(&mut view_state, item_data, context);
                    view_state
                }
                None => self.render_new_item(item_data, context),
            };

            if !lis {
                let next_sibling = self.end_flag_for_the_next_rendered_item_bottom_up.as_ref();
                self.parent_element
                    .insert_new_node_before_a_node(I::root_element(&view_state), next_sibling);
            }

            self.end_flag_for_the_next_rendered_item_bottom_up =
                Some(I::root_element(&view_state).web_sys_node().clone());
            *self
                .new_list
                .next_back()
                .expect_throw("keyed_list::KeyedListUpdater::update_other_items_in_the_middle") =
                Some(view_state);
        }
    }

    fn remove_items_still_in_old_list(&mut self) {
        let parent = self.parent_element;
        for (_, old_view_state) in self.old_list.by_ref() {
            let element = old_view_state
                .take()
                .expect_throw("keyed_list::KeyedListUpdater::remove_items_still_in_old_list");
            parent.remove_child(I::root_element(&element));
        }
    }

    fn insert_new_items_in_the_middle(
        &mut self,
        item_data: &mut PeekableDoubleEndedIterator<std::vec::IntoIter<&I>>,
        context: &Context<C>,
    ) {
        for item_data in item_data {
            let view_state = self.render_new_item(item_data, context);
            self.store_item_view_state(view_state);
        }
    }

    fn store_item_view_state(&mut self, view_state: I::ViewState) {
        *self
            .new_list
            .next()
            .expect_throw("keyed_list::KeyedListUpdater::store_item_view_state") = Some(view_state);
    }

    fn render_new_item(&self, item_data: &I, context: &Context<C>) -> I::ViewState {
        let mut view_state = I::create(self.template, item_data, context);
        I::update(&mut view_state, item_data, context);

        let next_sibling = self.end_flag_for_the_next_rendered_item_bottom_up.as_ref();
        self.parent_element
            .insert_new_node_before_a_node(I::root_element(&view_state), next_sibling);

        view_state
    }

    fn construct_old_items_map_from_remaining_old_items(&mut self) {
        self.old_items_map.clear();
        for (index, view_state) in self.old_list.by_ref() {
            let view_state = view_state.take().expect_throw(
                "keyed_list::KeyedListUpdater::construct_old_entries_map_from_remaining_old_entries",
            );
            let key = I::key_from_view_state(&view_state).clone();
            self.old_items_map
                .insert(key, OldItem { index, view_state });
        }
    }

    fn remove_old_items_that_still_in_old_items_map(&mut self) {
        let parent = self.parent_element;
        self.old_items_map.drain().for_each(|(_, item)| {
            parent.remove_child(I::root_element(&item.view_state));
        })
    }
}

struct ItemWithLis<I, VS> {
    item_data: I,
    old_view_state: Option<OldItem<VS>>,
    is_in_lis: bool,
}

impl<I, VS> ItemWithLis<I, VS> {
    fn new(item_data: I, old_view_state: Option<OldItem<VS>>) -> Self {
        Self {
            item_data,
            old_view_state,
            is_in_lis: false,
        }
    }
}

// Copied from https://github.com/axelf4/lis and modified to work with Spair.
fn longest_increasing_subsequence<I, VS>(entries: &mut [ItemWithLis<I, VS>]) {
    let mut p = vec![0; entries.len()];
    // indices of the new entries
    let mut m = Vec::with_capacity(entries.len());
    // only iter through entries with old index
    let mut it = entries
        .iter()
        .enumerate()
        .filter(|(_, x)| x.old_view_state.is_some());
    if let Some((i, _)) = it.next() {
        m.push(i);
    } else {
        return;
    }

    for (i, x) in it {
        // Test whether a[i] can extend the current sequence
        if entries[*m.last().unwrap_throw()]
            .old_view_state
            .as_ref()
            .unwrap_throw()
            .index
            .cmp(&x.old_view_state.as_ref().unwrap_throw().index)
            == std::cmp::Ordering::Less
        {
            p[i] = *m.last().unwrap_throw();
            m.push(i);
            continue;
        }

        // Binary search for largest j ≤ m.len() such that a[m[j]] < a[i]
        let j = match m.binary_search_by(|&j| {
            entries[j]
                .old_view_state
                .as_ref()
                .unwrap_throw()
                .index
                .cmp(&x.old_view_state.as_ref().unwrap_throw().index)
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
        entries[k].is_in_lis = true;
        k = p[k];
    }
}

// These things are copied from https://github.com/axelf4/lis
// because Spair unable to used `lis` without changes.
pub struct PeekableDoubleEndedIterator<I: Iterator> {
    iter: I,
    peeked_front: Option<Option<I::Item>>,
    peeked_back: Option<Option<I::Item>>,
}

pub trait PeekableDoubleEnded: Sized + Iterator {
    fn peekable_double_ended(self) -> PeekableDoubleEndedIterator<Self> {
        PeekableDoubleEndedIterator {
            iter: self,
            peeked_front: None,
            peeked_back: None,
        }
    }
}

impl<T: Iterator> PeekableDoubleEnded for T {}

impl<I: Iterator> PeekableDoubleEndedIterator<I> {
    #[inline]
    pub fn peek(&mut self) -> Option<&I::Item> {
        if self.peeked_front.is_none() {
            self.peeked_front = Some(
                self.iter
                    .next()
                    .or_else(|| self.peeked_back.take().unwrap_or(None)),
            );
        }
        match self.peeked_front {
            Some(Some(ref value)) => Some(value),
            Some(None) => None,
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn peek_back(&mut self) -> Option<&I::Item>
    where
        I: DoubleEndedIterator,
    {
        if self.peeked_back.is_none() {
            self.peeked_back = Some(
                self.iter
                    .next_back()
                    .or_else(|| self.peeked_front.take().unwrap_or(None)),
            );
        }
        match self.peeked_back {
            Some(Some(ref value)) => Some(value),
            Some(None) => None,
            _ => unreachable!(),
        }
    }
}

impl<I: Iterator> Iterator for PeekableDoubleEndedIterator<I> {
    type Item = I::Item;
    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        self.peeked_front
            .take()
            .unwrap_or_else(|| self.iter.next())
            .or_else(|| self.peeked_back.take().unwrap_or(None))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let peek_len = match self.peeked_front {
            Some(None) => return (0, Some(0)),
            Some(Some(_)) => 1,
            None => 0,
        } + match self.peeked_back {
            Some(None) => return (0, Some(0)),
            Some(Some(_)) => 1,
            None => 0,
        };
        let (lo, hi) = self.iter.size_hint();
        (
            lo.saturating_add(peek_len),
            hi.and_then(|x| x.checked_add(peek_len)),
        )
    }
}

impl<I: DoubleEndedIterator> DoubleEndedIterator for PeekableDoubleEndedIterator<I> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.peeked_back
            .take()
            .unwrap_or_else(|| self.iter.next_back())
            .or_else(|| self.peeked_front.take().unwrap_or(None))
    }
}

#[cfg(test)]
pub mod lis_tests {
    use wasm_bindgen_test::*;
    impl super::ItemWithLis<(), ()> {
        fn index(index: usize) -> Self {
            Self {
                item_data: (),
                old_view_state: Some(super::OldItem {
                    index,
                    view_state: (),
                }),
                is_in_lis: false,
            }
        }
        fn none() -> Self {
            Self {
                item_data: (),
                old_view_state: None,
                is_in_lis: false,
            }
        }
    }

    fn collect_lis(mut entries: Vec<super::ItemWithLis<(), ()>>) -> Vec<usize> {
        super::longest_increasing_subsequence(&mut entries[..]);
        entries
            .iter()
            .flat_map(|entry| {
                if entry.is_in_lis {
                    entry
                        .old_view_state
                        .as_ref()
                        .map(|old_entry| old_entry.index)
                } else {
                    None
                }
            })
            .collect()
    }

    fn collect_lis_from_index(indices: &[usize]) -> Vec<usize> {
        let entries = indices
            .iter()
            .map(|i| super::ItemWithLis::index(*i))
            .collect();
        collect_lis(entries)
    }

    #[wasm_bindgen_test]
    fn lis_with_none() {
        let entries = vec![
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
        let rs = collect_lis(entries);
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
}

#[cfg(test)]
pub mod keyed_list_tests {
    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::Node;

    use crate::{
        Element, Text,
        test_helper::{self, TestComp, TestDataInterface},
    };

    use super::{KeyedList, KeyedListItemView};

    type TestData = Vec<&'static str>;
    type TestState = TestComp<TestData>;

    pub struct TestDataViewState {
        keyed_list: KeyedList<TestState, &'static str>,
    }

    pub struct TestItemViewState {
        data: &'static str,
        element: Element,
        text: Text,
    }

    impl KeyedListItemView<TestState> for &'static str {
        type ViewState = TestItemViewState;

        type Key = &'static str;

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
                data: item_data,
                element,
                text,
            }
        }

        fn update(
            view_state: &mut Self::ViewState,
            item_data: &Self,
            _context: &crate::Context<TestState>,
        ) {
            view_state.text.update(*item_data);
        }

        fn root_element(view_state: &Self::ViewState) -> &crate::WsElement {
            &view_state.element
        }
    }

    impl TestDataInterface for TestData {
        type ViewState = TestDataViewState;

        fn init(&self, root: &Element, context: &crate::Context<TestState>) -> Self::ViewState {
            let mut keyed_list = KeyedList::new(&root.ws_element(), None);
            keyed_list.update(self.iter(), context);
            TestDataViewState { keyed_list }
        }

        fn update(&self, view_state: &mut Self::ViewState, context: &crate::Context<TestState>) {
            view_state.keyed_list.update(self.iter(), &context);
        }
    }

    fn collect_text_from_child_nodes(root_node: &Node) -> Vec<String> {
        let mut list = Vec::new();
        let mut maybe_node = root_node.first_child();
        while let Some(node) = maybe_node {
            if let Some(text) = node.text_content() {
                list.push(text);
            }
            maybe_node = node.next_sibling();
        }
        list
    }

    #[wasm_bindgen_test]
    fn keyed_list() {
        let test = test_helper::Test::set_up(Vec::new());
        assert_eq!(Some(""), test.text_content().as_deref());
        let empty: Vec<&'static str> = Vec::new();
        test.update(empty.clone());
        assert_eq!(Some(""), test.text_content().as_deref());
        assert_eq!(
            empty,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );

        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"];
        test.update(data.clone());
        assert_eq!(
            data,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );
        assert_eq!(Some("abcdefghijk"), test.text_content().as_deref());

        // Random shuffle + addition
        let data = vec!["f", "b", "d", "l", "g", "i", "m", "j", "a", "h", "k"];
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
        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"];
        test.update(data.clone());
        assert_eq!(
            data,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );
        assert_eq!(Some("abcdefghijk"), test.text_content().as_deref());

        // Forward
        let data = vec!["a", "i", "b", "c", "d", "e", "f", "g", "h", "j", "k"];
        test.update(data.clone());
        assert_eq!(
            data,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );
        assert_eq!(Some("aibcdefghjk"), test.text_content().as_deref());

        // Backward
        let data = vec!["a", "i", "c", "d", "e", "f", "g", "h", "b", "j", "k"];
        test.update(data.clone());
        assert_eq!(
            data,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );
        assert_eq!(Some("aicdefghbjk"), test.text_content().as_deref());

        // Swap
        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"];
        test.update(data.clone());
        assert_eq!(
            data,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );
        assert_eq!(Some("abcdefghijk"), test.text_content().as_deref());

        // Remove middle
        let data = vec!["a", "b", "c", "d", "i", "j", "k"];
        test.update(data.clone());
        assert_eq!(
            data,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );
        assert_eq!(Some("abcdijk"), test.text_content().as_deref());

        // Insert middle
        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"];
        test.update(data.clone());
        assert_eq!(
            data,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );
        assert_eq!(Some("abcdefghijk"), test.text_content().as_deref());

        // Remove start
        let data = vec!["d", "e", "f", "g", "h", "i", "j", "k"];
        test.update(data.clone());
        assert_eq!(
            data,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );
        assert_eq!(Some("defghijk"), test.text_content().as_deref());

        // Insert start
        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"];
        test.update(data.clone());
        assert_eq!(
            data,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );
        assert_eq!(Some("abcdefghijk"), test.text_content().as_deref());

        // Remove end
        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h"];
        test.update(data.clone());
        assert_eq!(
            data,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );
        assert_eq!(Some("abcdefgh"), test.text_content().as_deref());

        // Append end
        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"];
        test.update(data.clone());
        assert_eq!(
            data,
            test.execute_on_root_node(collect_text_from_child_nodes)
        );
        assert_eq!(Some("abcdefghijk"), test.text_content().as_deref());
    }
}
