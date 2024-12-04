use std::{collections::HashMap, hash::Hash, iter::Enumerate, slice::IterMut};

use wasm_bindgen::UnwrapThrowExt;

use crate::{Component, Context, TemplateElement, WsElement};

pub trait KeyedItemRender<C: Component> {
    type Item: PartialEq;
    type Key: Clone + Eq + Hash;
    fn template_string() -> &'static str;
    fn key(&self) -> &Self::Key;
    fn key_from_state(state: &Self::Item) -> &Self::Key;
    fn create(item: &Self::Item, template: &TemplateElement, context: &Context<C>) -> Self;
    fn update(&mut self, item: &Self::Item, context: &Context<C>);
    fn root_element(&self) -> &WsElement;
}

pub struct KeyedList<C, U>
where
    U: KeyedItemRender<C>,
    C: Component,
{
    parent_element: WsElement,
    template: TemplateElement,
    active_items: Vec<Option<U>>,
    buffer_items: Vec<Option<U>>,
    active_items_map: HashMap<U::Key, OldItem<U>>,
}

struct KeyedListUpdater<'a, C, U>
where
    U: KeyedItemRender<C>,
    C: Component,
{
    parent_element: &'a WsElement,
    template: &'a TemplateElement,
    old_list: PeekableDoubleEndedIterator<Enumerate<IterMut<'a, Option<U>>>>,
    new_list: PeekableDoubleEndedIterator<IterMut<'a, Option<U>>>,
    old_items_map: &'a mut HashMap<U::Key, OldItem<U>>,
    end_flag_for_the_next_entry_bottom_up: Option<WsElement>,
}

pub struct OldItem<U> {
    pub index: usize,
    pub updater: U,
}

impl<C, U> KeyedList<C, U>
where
    U: KeyedItemRender<C> + 'static,
    C: Component + 'static,
{
    pub fn new(parent_element: WsElement) -> Self {
        Self {
            parent_element,
            template: TemplateElement::new(U::template_string()),
            active_items: Vec::new(),
            buffer_items: Vec::new(),
            active_items_map: HashMap::new(),
        }
    }

    pub fn update<'a>(&mut self, items: impl Iterator<Item = &'a U::Item>, context: Context<C>) {
        // Current implementation requires knowing the exact number in advance.
        let items: Vec<_> = items.collect();

        let new_count = items.len();
        if new_count == 0 {
            self.remove_all_old_items();
            return;
        }

        self.active_items_map.reserve(new_count);
        if new_count < self.buffer_items.len() {
            self.buffer_items.truncate(new_count);
        } else {
            self.buffer_items
                .extend((0..(new_count - self.buffer_items.len())).map(|_| None));
        }
        std::mem::swap(&mut self.active_items, &mut self.buffer_items);

        log::info!(
            "old list {}, new list {}",
            self.buffer_items.len(),
            self.active_items.len()
        );

        let mut updater = KeyedListUpdater {
            parent_element: &self.parent_element,
            template: &self.template,
            old_list: self
                .buffer_items
                .iter_mut()
                .enumerate()
                .peekable_double_ended(),
            new_list: self.active_items.iter_mut().peekable_double_ended(),
            old_items_map: &mut self.active_items_map,
            end_flag_for_the_next_entry_bottom_up: None,
        };
        updater.update(items, context);
        log::info!(
            "old list {}, new list {}",
            self.buffer_items.len(),
            self.active_items.len()
        );
    }

    fn remove_all_old_items(&mut self) {
        self.parent_element.clear_text_content();
        for item in self.active_items.iter_mut() {
            item.take();
        }
    }
}

impl<'a, C, U> KeyedListUpdater<'a, C, U>
where
    U: KeyedItemRender<C> + 'static,
    C: Component + 'static,
{
    fn update(&mut self, items: Vec<&U::Item>, context: Context<C>) {
        let mut items = items.into_iter().peekable_double_ended();
        loop {
            let mut count = self.update_same_items_from_start(&mut items, &context);
            count += self.update_same_items_from_end(&mut items, &context);
            count += self.update_moved_forward_item(&mut items, &context);
            count += self.update_moved_backward_item(&mut items, &context);
            if count == 0 {
                break;
            }
        }
        self.update_items_in_the_middle(&mut items, &context);
    }

    fn update_same_items_from_start(
        &mut self,
        items: &mut PeekableDoubleEndedIterator<std::vec::IntoIter<&U::Item>>,
        context: &Context<C>,
    ) -> usize {
        let mut count = 0;
        loop {
            // Check if the first items from `items` and `old_list` have the same key.
            if let (Some(item_state), Some(old_item)) = (items.peek(), self.old_list.peek()) {
                let old_entry = old_item
                    .1
                    .as_ref()
                    .expect_throw("keyed_list::KeyedListUpdater::update_same_items_from_start");
                if !U::key_from_state(item_state).eq(old_entry.key()) {
                    return count;
                }
            } else {
                return count;
            }

            // Yes, they have the same key, then update the old item
            count += 1;
            Self::update_existing_item(
                self.parent_element,
                items.next().unwrap_throw(),
                self.old_list.next(),
                self.new_list.next(),
                self.old_list.peek().and_then(|next_old_item| {
                    next_old_item
                        .1
                        .as_ref()
                        .map(|updater| updater.root_element())
                }),
                false,
                context,
            );
        }
    }

    fn update_existing_item(
        parent_element: &WsElement,
        item_state: &U::Item,
        old_item: Option<(usize, &mut Option<U>)>,
        new_item: Option<&mut Option<U>>,
        next_sibling: Option<&WsElement>,
        relocate_item: bool,
        context: &Context<C>,
    ) {
        let mut old_item = old_item.unwrap_throw().1.take().unwrap_throw();
        if relocate_item {
            parent_element.insert_new_node_before_a_node(old_item.root_element(), next_sibling);
        }
        old_item.update(item_state, context);
        if let Some(new_item) = new_item {
            *new_item = Some(old_item);
        }
    }

    fn update_same_items_from_end(
        &mut self,
        items: &mut PeekableDoubleEndedIterator<std::vec::IntoIter<&U::Item>>,
        context: &Context<C>,
    ) -> usize {
        let mut count = 0;
        loop {
            let new_end_flag = if let (Some(item_state), Some(old_item)) =
                (items.peek_back(), self.old_list.peek_back())
            {
                let old_item = old_item
                    .1
                    .as_ref()
                    .expect_throw("keyed_list::KeyedListUpdater::update_same_items_from_end");

                if !U::key_from_state(item_state).eq(old_item.key()) {
                    return count;
                }
                // The keys are the same, so we need to continue the loop.
                // The proccessed item  will be upped by one from the bottom.
                // And we will need to update `end_flag_for_the_next_entry_bottom_up`
                // with this value
                old_item.root_element().clone()
            } else {
                return count;
            };
            count += 1;
            Self::update_existing_item(
                self.parent_element,
                items.next_back().unwrap_throw(),
                self.old_list.next_back(),
                self.new_list.next_back(),
                self.end_flag_for_the_next_entry_bottom_up.as_ref(),
                false,
                context,
            );
            self.end_flag_for_the_next_entry_bottom_up = Some(new_end_flag);
        }
    }

    fn update_moved_forward_item(
        &mut self,
        items: &mut PeekableDoubleEndedIterator<std::vec::IntoIter<&U::Item>>,
        context: &Context<C>,
    ) -> usize {
        if let (Some(item_state), Some(old_item)) = (items.peek(), self.old_list.peek_back()) {
            let old_item = old_item
                .1
                .as_ref()
                .expect_throw("keyed_list::KeyedListUpdater::update_moved_forward_item");
            if !U::key_from_state(item_state).eq(old_item.key()) {
                // No entry moved forward
                return 0;
            }
        } else {
            return 0;
        }

        let moved = self.old_list.next_back();
        let next_sibling = self
            .old_list
            .peek()
            .and_then(|entry| entry.1.as_ref().map(|item| item.root_element()));
        Self::update_existing_item(
            self.parent_element,
            items.next().unwrap_throw(),
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
        items: &mut PeekableDoubleEndedIterator<std::vec::IntoIter<&U::Item>>,
        context: &Context<C>,
    ) -> usize {
        let new_end_flag =
            if let (Some(item_state), Some(old_item)) = (items.peek_back(), self.old_list.peek()) {
                let old_item = old_item
                    .1
                    .as_ref()
                    .expect_throw("keyed_list::KeyedListUpdater::update_moved_backward_item");
                if !U::key_from_state(item_state).eq(old_item.key()) {
                    // No entry moved backward
                    return 0;
                }
                old_item.root_element().clone()
            } else {
                return 0;
            };
        Self::update_existing_item(
            self.parent_element,
            items.next_back().unwrap_throw(),
            self.old_list.next(),
            self.new_list.next_back(),
            self.end_flag_for_the_next_entry_bottom_up.as_ref(),
            true,
            context,
        );
        self.end_flag_for_the_next_entry_bottom_up = Some(new_end_flag);
        1
    }

    fn update_items_in_the_middle(
        &mut self,
        items: &mut PeekableDoubleEndedIterator<std::vec::IntoIter<&U::Item>>,
        context: &Context<C>,
    ) {
        if items.peek().is_none() {
            self.remove_items_still_in_old_list();
            return;
        }

        if self.old_list.peek().is_none() {
            self.insert_new_items_in_the_middle(items, context);
            return;
        }

        self.construct_old_items_map_from_remaining_old_items();

        // Using longest_increasing_subsequence find which elements should be moved around in the browser's DOM
        // and which should be stay still
        let mut entries_with_lis: Vec<_> = items
            .map(|item_state| {
                let key = U::key_from_state(item_state);
                let old_item = self.old_items_map.remove(key);
                ItemWithLis::new(item_state, old_item)
            })
            .collect();
        longest_increasing_subsequence(&mut entries_with_lis);

        self.remove_old_items_that_still_in_old_items_map();

        for iwl in entries_with_lis.into_iter().rev() {
            let ItemWithLis {
                item_state,
                old_item,
                lis,
            } = iwl;

            let updater = match old_item {
                Some(old_entry) => {
                    let mut updater = old_entry.updater;
                    updater.update(item_state, context);
                    updater
                }
                None => U::create(item_state, self.template, context),
            };

            let next_sibling = self.end_flag_for_the_next_entry_bottom_up.as_ref();
            if !lis {
                self.parent_element
                    .insert_new_node_before_a_node(updater.root_element(), next_sibling);
            }

            self.end_flag_for_the_next_entry_bottom_up = Some(updater.root_element().clone());
            *self
                .new_list
                .next_back()
                .expect_throw("keyed_list::KeyedListUpdater::update_other_items_in_the_middle") =
                Some(updater);
        }
    }

    fn remove_items_still_in_old_list(&mut self) {
        let parent = self.parent_element;
        for (_, item) in self.old_list.by_ref() {
            let element = item
                .take()
                .expect_throw("keyed_list::KeyedListUpdater::remove_remain_entries");
            parent.remove_child(element.root_element());
        }
    }

    fn insert_new_items_in_the_middle(
        &mut self,
        items: &mut PeekableDoubleEndedIterator<std::vec::IntoIter<&U::Item>>,
        context: &Context<C>,
    ) {
        for entry_state in items {
            let updater = self.render_an_item(entry_state, context);
            self.store_item_updater(updater);
        }
    }

    fn store_item_updater(&mut self, updater: U) {
        *self
            .new_list
            .next()
            .expect_throw("keyed_list::KeyedListUpdater::store_item_updater") = Some(updater);
    }

    fn render_an_item(&self, item_state: &U::Item, context: &Context<C>) -> U {
        let updater = U::create(item_state, self.template, context);

        let next_sibling = self.end_flag_for_the_next_entry_bottom_up.as_ref();
        self.parent_element
            .insert_new_node_before_a_node(updater.root_element(), next_sibling);

        updater
    }

    fn construct_old_items_map_from_remaining_old_items(&mut self) {
        self.old_items_map.clear();
        for (index, item) in self.old_list.by_ref() {
            let updater = item.take().expect_throw(
                "keyed_list::KeyedListUpdater::construct_old_entries_map_from_remaining_old_entries",
            );
            let key = updater.key().clone();
            self.old_items_map.insert(key, OldItem { index, updater });
        }
    }

    fn remove_old_items_that_still_in_old_items_map(&mut self) {
        let parent = self.parent_element;
        self.old_items_map.drain().for_each(|(_, item)| {
            parent.remove_child(item.updater.root_element());
        })
    }
}

struct ItemWithLis<I, U> {
    item_state: I,
    old_item: Option<OldItem<U>>,
    lis: bool,
}

impl<I, U> ItemWithLis<I, U> {
    fn new(item_state: I, old_item: Option<OldItem<U>>) -> Self {
        Self {
            item_state,
            old_item,
            lis: false,
        }
    }
}

// Copied from https://github.com/axelf4/lis and modified to work with Spair.
fn longest_increasing_subsequence<I, U>(entries: &mut [ItemWithLis<I, U>]) {
    let mut p = vec![0; entries.len()];
    // indices of the new entries
    let mut m = Vec::with_capacity(entries.len());
    // only iter through entries with old index
    let mut it = entries
        .iter()
        .enumerate()
        .filter(|(_, x)| x.old_item.is_some());
    if let Some((i, _)) = it.next() {
        m.push(i);
    } else {
        return;
    }

    for (i, x) in it {
        // Test whether a[i] can extend the current sequence
        if entries[*m.last().unwrap_throw()]
            .old_item
            .as_ref()
            .unwrap_throw()
            .index
            .cmp(&x.old_item.as_ref().unwrap_throw().index)
            == std::cmp::Ordering::Less
        {
            p[i] = *m.last().unwrap_throw();
            m.push(i);
            continue;
        }

        // Binary search for largest j ≤ m.len() such that a[m[j]] < a[i]
        let j = match m.binary_search_by(|&j| {
            entries[j]
                .old_item
                .as_ref()
                .unwrap_throw()
                .index
                .cmp(&x.old_item.as_ref().unwrap_throw().index)
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
        entries[k].lis = true;
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
pub mod keyed_list_tests {
    use wasm_bindgen_test::*;
    impl super::ItemWithLis<(), ()> {
        fn index(index: usize) -> Self {
            Self {
                item_state: (),
                old_item: Some(super::OldItem { index, updater: () }),
                lis: false,
            }
        }
        fn none() -> Self {
            Self {
                item_state: (),
                old_item: None,
                lis: false,
            }
        }
    }

    fn collect_lis(mut entries: Vec<super::ItemWithLis<(), ()>>) -> Vec<usize> {
        super::longest_increasing_subsequence(&mut entries[..]);
        entries
            .iter()
            .flat_map(|entry| {
                if entry.lis {
                    entry.old_item.as_ref().map(|old_entry| old_entry.index)
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
