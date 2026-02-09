use std::{hash::Hash, iter::Enumerate, marker::PhantomData, slice::IterMut};

// use rustc_hash::FxHashMap as HashMap;
use std::collections::HashMap;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::DocumentFragment;

use crate::{TemplateElement, WsElement, WsNodeFns};

pub struct KeyedList<K, VS>
where
    K: Clone + Eq + Hash,
{
    parent_element: WsElement,
    template: TemplateElement,

    active_items: Vec<Option<KeyVS<K, VS>>>,
    buffer_items: Vec<Option<KeyVS<K, VS>>>,
    active_items_map: HashMap<K, OldItem<VS>>,

    end_node_marker_for_partial_list: Option<web_sys::Node>,
}

struct KeyVS<K, VS> {
    key: K,
    vs: VS,
}

struct OldItem<VS> {
    pub index: usize,
    pub view_state: VS,
}

pub trait ItemViewState {
    fn root_element(&self) -> &WsElement;
}

struct KeyedListUpdater<'a, I, K, VS, GK, CV, UV>
where
    K: Clone + Eq + Hash,
    VS: ItemViewState,
    GK: Fn(&I) -> &K,
    CV: Fn(DocumentFragment, &I) -> VS,
    UV: Fn(&mut VS, I),
{
    parent_element: &'a WsElement,
    template: &'a TemplateElement,

    get_key_fn: GK,
    create_view_fn: CV,
    update_view_fn: UV,

    #[allow(clippy::type_complexity)]
    old_list: PeekableDoubleEndedIterator<Enumerate<IterMut<'a, Option<KeyVS<K, VS>>>>>,
    new_list: PeekableDoubleEndedIterator<IterMut<'a, Option<KeyVS<K, VS>>>>,
    old_items_map: &'a mut HashMap<K, OldItem<VS>>,
    end_flag_for_the_next_rendered_item_bottom_up: Option<web_sys::Node>,

    _iki: PhantomData<I>,
}

impl<K, VS> KeyedList<K, VS>
where
    K: Clone + Eq + Hash,
    VS: ItemViewState,
{
    pub fn new(
        parent_element: &WsElement,
        end_node_marker_for_partial_list: Option<web_sys::Node>,
        template_string: &str,
    ) -> Self {
        Self {
            parent_element: parent_element.clone(),
            template: TemplateElement::new(template_string),
            end_node_marker_for_partial_list,

            active_items: Vec::new(),
            buffer_items: Vec::new(),
            active_items_map: HashMap::default(),
        }
    }

    pub fn end_node(&self) -> Option<&web_sys::Node> {
        self.end_node_marker_for_partial_list.as_ref()
    }

    pub fn update<I, GK, CV, UV>(
        &mut self,
        item_data: impl Iterator<Item = I>,
        get_key_fn: GK,
        create_view_fn: CV,
        update_view_fn: UV,
    ) where
        GK: Fn(&I) -> &K,
        CV: Fn(DocumentFragment, &I) -> VS,
        UV: Fn(&mut VS, I),
    {
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

            get_key_fn,
            create_view_fn,
            update_view_fn,

            end_flag_for_the_next_rendered_item_bottom_up: self
                .end_node_marker_for_partial_list
                .clone(),

            _iki: PhantomData,
        };
        keyed_list_updater.update(item_data);
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
                    self.parent_element.remove_child(item.vs.root_element());
                };
            }
        }
    }
}

impl<I, K, VS, GK, CV, UV> KeyedListUpdater<'_, I, K, VS, GK, CV, UV>
where
    K: Clone + Eq + Hash,
    VS: ItemViewState,
    GK: Fn(&I) -> &K,
    CV: Fn(DocumentFragment, &I) -> VS,
    UV: Fn(&mut VS, I),
{
    fn update(&mut self, item_data: Vec<I>) {
        let mut item_data = item_data.into_iter().peekable_double_ended();
        let mut total_count = 0;
        loop {
            let mut count = self.update_same_items_from_start(&mut item_data);
            count += self.update_same_items_from_end(&mut item_data);
            count += self.update_moved_forward_item(&mut item_data);
            count += self.update_moved_backward_item(&mut item_data);
            if count == 0 {
                break;
            }
            total_count += count;
        }
        self.update_items_in_the_middle(&mut item_data, total_count == 0);
    }

    fn update_same_items_from_start(
        &mut self,
        item_data: &mut PeekableDoubleEndedIterator<std::vec::IntoIter<I>>,
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
                if !old_view_state.key.eq((self.get_key_fn)(item_data)) {
                    return count;
                }
            } else {
                return count;
            }

            // Yes, they have the same key, then update the old item
            count += 1;
            Self::update_existing_item(
                self.parent_element,
                &self.update_view_fn,
                item_data.next().unwrap_throw(),
                self.old_list.next(),
                self.new_list.next(),
                None,
                false,
            );
        }
    }

    fn update_existing_item(
        parent_element: &WsElement,
        update_view_fn: &UV,
        item_data: I,
        old_view_state: Option<(usize, &mut Option<KeyVS<K, VS>>)>,
        new_view_state: Option<&mut Option<KeyVS<K, VS>>>,
        next_sibling: Option<&web_sys::Node>,
        relocating_item: bool,
    ) {
        let mut old_view_state = old_view_state.unwrap_throw().1.take().unwrap_throw();
        if relocating_item {
            parent_element
                .insert_new_node_before_a_node(old_view_state.vs.root_element(), next_sibling);
        }
        update_view_fn(&mut old_view_state.vs, item_data);
        if let Some(new_view_state) = new_view_state {
            *new_view_state = Some(old_view_state);
        }
    }

    fn update_same_items_from_end(
        &mut self,
        item_data: &mut PeekableDoubleEndedIterator<std::vec::IntoIter<I>>,
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

                if !old_view_state.key.eq((self.get_key_fn)(item_data)) {
                    return count;
                }
                // The keys are the same, so we need to continue the loop.
                // The proccessed item  will be upped by one from the bottom.
                // And we will need to update `end_flag_for_the_next_entry_bottom_up`
                // with this value
                old_view_state.vs.root_element().get_ws_node_ref().clone()
            } else {
                return count;
            };
            count += 1;
            Self::update_existing_item(
                self.parent_element,
                &self.update_view_fn,
                item_data.next_back().unwrap_throw(),
                self.old_list.next_back(),
                self.new_list.next_back(),
                None,
                false,
            );
            self.end_flag_for_the_next_rendered_item_bottom_up = Some(new_end_flag);
        }
    }

    fn update_moved_forward_item(
        &mut self,
        item_data: &mut PeekableDoubleEndedIterator<std::vec::IntoIter<I>>,
    ) -> usize {
        if let (Some(item_data), Some(old_view_state)) =
            (item_data.peek(), self.old_list.peek_back())
        {
            let old_view_state = old_view_state
                .1
                .as_ref()
                .expect_throw("keyed_list::KeyedListUpdater::update_moved_forward_item");
            if !old_view_state.key.eq((self.get_key_fn)(item_data)) {
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
                .map(|view_state| view_state.vs.root_element().get_ws_node_ref())
        });
        Self::update_existing_item(
            self.parent_element,
            &self.update_view_fn,
            item_data.next().unwrap_throw(),
            moved,
            self.new_list.next(),
            next_sibling,
            true,
        );
        1
    }

    fn update_moved_backward_item(
        &mut self,
        item_data: &mut PeekableDoubleEndedIterator<std::vec::IntoIter<I>>,
    ) -> usize {
        let new_end_flag = if let (Some(item_data), Some(old_view_state)) =
            (item_data.peek_back(), self.old_list.peek())
        {
            let old_view_state = old_view_state
                .1
                .as_ref()
                .expect_throw("keyed_list::KeyedListUpdater::update_moved_backward_item");
            if !old_view_state.key.eq((self.get_key_fn)(item_data)) {
                // No entry moved backward
                return 0;
            }
            old_view_state.vs.root_element().get_ws_node_ref().clone()
        } else {
            return 0;
        };
        Self::update_existing_item(
            self.parent_element,
            &self.update_view_fn,
            item_data.next_back().unwrap_throw(),
            self.old_list.next(),
            self.new_list.next_back(),
            self.end_flag_for_the_next_rendered_item_bottom_up.as_ref(),
            true,
        );
        self.end_flag_for_the_next_rendered_item_bottom_up = Some(new_end_flag);
        1
    }

    fn update_items_in_the_middle(
        &mut self,
        item_data: &mut PeekableDoubleEndedIterator<std::vec::IntoIter<I>>,
        no_items_render_yet: bool,
    ) {
        // No more items, remove all old items
        if item_data.peek().is_none() {
            self.remove_items_still_in_old_list();
            return;
        }

        // No more old items, all available items are new
        if self.old_list.peek().is_none() {
            self.insert_new_items_in_the_middle(item_data);
            return;
        }

        self.construct_old_items_map_from_remaining_old_items();

        // Using longest_increasing_subsequence to find which elements should be moved around in the browser's DOM
        // and which should be stay still
        let mut items_has_old_view_state = 0;
        let mut item_data_with_lis: Vec<_> = item_data
            .map(|item_data| {
                let key = (self.get_key_fn)(&item_data);
                let old_view_state = self.old_items_map.remove(key);
                if old_view_state.is_some() {
                    items_has_old_view_state += 1;
                }
                ItemWithLis::new(key.clone(), item_data, old_view_state)
            })
            .collect();
        longest_increasing_subsequence(&mut item_data_with_lis);

        let all_items_are_new = no_items_render_yet && items_has_old_view_state == 0;
        self.remove_old_items_that_still_in_old_items_map(all_items_are_new);
        if all_items_are_new {
            self.just_update_items_from_start_to_end(item_data_with_lis);
        } else {
            self.update_items_with_lis(item_data_with_lis);
        }
    }

    fn just_update_items_from_start_to_end(
        &mut self,
        item_data_with_lis: Vec<ItemWithLis<K, I, VS>>,
    ) {
        for item_data_with_lis in item_data_with_lis {
            let key = item_data_with_lis.key;
            let view_state = self.render_new_item(item_data_with_lis.item_data);
            self.store_item_view_state(key, view_state);
        }
    }

    fn update_items_with_lis(&mut self, item_data_with_lis: Vec<ItemWithLis<K, I, VS>>) {
        for iwl in item_data_with_lis.into_iter().rev() {
            let ItemWithLis {
                key,
                item_data,
                old_view_state,
                is_in_lis: lis,
            } = iwl;

            let view_state = match old_view_state {
                Some(old) => {
                    let mut view_state = old.view_state;
                    (self.update_view_fn)(&mut view_state, item_data);
                    view_state
                }
                None => self.render_new_item(item_data),
            };

            if !lis {
                let next_sibling = self.end_flag_for_the_next_rendered_item_bottom_up.as_ref();
                self.parent_element
                    .insert_new_node_before_a_node(view_state.root_element(), next_sibling);
            }

            self.end_flag_for_the_next_rendered_item_bottom_up =
                Some(view_state.root_element().get_ws_node_ref().clone());
            *self
                .new_list
                .next_back()
                .expect_throw("keyed_list::KeyedListUpdater::update_other_items_in_the_middle") =
                Some(KeyVS {
                    key,
                    vs: view_state,
                });
        }
    }

    fn remove_items_still_in_old_list(&mut self) {
        for (_, old_view_state) in self.old_list.by_ref() {
            let item = old_view_state
                .take()
                .expect_throw("keyed_list::KeyedListUpdater::remove_items_still_in_old_list");
            self.parent_element.remove_child(item.vs.root_element());
        }
    }

    fn insert_new_items_in_the_middle(
        &mut self,
        item_data: &mut PeekableDoubleEndedIterator<std::vec::IntoIter<I>>,
    ) {
        for item_data in item_data {
            let key = Clone::clone((self.get_key_fn)(&item_data));
            let view_state = self.render_new_item(item_data);
            self.store_item_view_state(key, view_state);
        }
    }

    fn store_item_view_state(&mut self, key: K, view_state: VS) {
        *self
            .new_list
            .next()
            .expect_throw("keyed_list::KeyedListUpdater::store_item_view_state") = Some(KeyVS {
            key,
            vs: view_state,
        });
    }

    fn render_new_item(&self, item_data: I) -> VS {
        let mut view_state = (self.create_view_fn)(self.template.fragment_clone(), &item_data);
        (self.update_view_fn)(&mut view_state, item_data);

        let next_sibling = self.end_flag_for_the_next_rendered_item_bottom_up.as_ref();
        self.parent_element
            .insert_new_node_before_a_node(view_state.root_element(), next_sibling);

        view_state
    }

    fn construct_old_items_map_from_remaining_old_items(&mut self) {
        self.old_items_map.clear();
        for (index, view_state) in self.old_list.by_ref() {
            let view_state = view_state.take().expect_throw(
                "keyed_list::KeyedListUpdater::construct_old_entries_map_from_remaining_old_entries",
            );
            self.old_items_map.insert(
                view_state.key,
                OldItem {
                    index,
                    view_state: view_state.vs,
                },
            );
        }
    }

    fn remove_old_items_that_still_in_old_items_map(&mut self, all_items_are_new: bool) {
        if all_items_are_new && self.end_flag_for_the_next_rendered_item_bottom_up.is_none() {
            self.parent_element.clear_text_content();
            self.old_items_map.clear();
        } else {
            for (_, item) in self.old_items_map.drain() {
                self.parent_element
                    .remove_child(item.view_state.root_element());
            }
        }
    }
}

struct ItemWithLis<K, I, VS> {
    key: K,
    item_data: I,
    old_view_state: Option<OldItem<VS>>,
    is_in_lis: bool,
}

impl<K, I, VS> ItemWithLis<K, I, VS> {
    fn new(key: K, item_data: I, old_view_state: Option<OldItem<VS>>) -> Self {
        Self {
            key,
            item_data,
            old_view_state,
            is_in_lis: false,
        }
    }
}

// Copied from https://github.com/axelf4/lis and modified to work with Spair.
fn longest_increasing_subsequence<K, I, VS>(entries: &mut [ItemWithLis<K, I, VS>]) {
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
