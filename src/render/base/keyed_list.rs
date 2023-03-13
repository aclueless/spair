use wasm_bindgen::UnwrapThrowExt;

use super::NodesUpdater;
use crate::{
    component::{Comp, Component},
    dom::{
        ElementStatus, GroupedNodes, KeyedEntry, KeyedList, ListEntryKey, ListEntryTemplate,
        OldEntry,
    },
};

pub struct KeyedListContext<'a> {
    parent: &'a web_sys::Node,
    old: PeekableDoubleEndedIterator<
        std::iter::Enumerate<std::slice::IterMut<'a, Option<KeyedEntry>>>,
    >,
    new: PeekableDoubleEndedIterator<std::slice::IterMut<'a, Option<KeyedEntry>>>,
    old_entries_map: &'a mut std::collections::HashMap<ListEntryKey, OldEntry>,
    new_entry_count: usize,
    end_flag_for_the_next_entry_bottom_up: Option<web_sys::Node>,
    template: Option<&'a mut ListEntryTemplate>,
    require_init_template: bool,
}

impl<'a> KeyedListContext<'a> {
    pub fn new(
        list: &'a mut KeyedList,
        new_entry_count: usize,
        parent: &'a web_sys::Node,
        use_template: bool,
    ) -> Self {
        list.pre_update(new_entry_count);

        let require_init_template = match use_template {
            true => list.require_init_template(),
            false => false,
        };

        let (template, old, new, old_entries_map) = list.entries_mut();
        KeyedListContext {
            parent,
            old: old.iter_mut().enumerate().peekable_double_ended(),
            new: new.iter_mut().peekable_double_ended(),
            old_entries_map,
            new_entry_count,
            end_flag_for_the_next_entry_bottom_up: None,
            template,
            require_init_template,
        }
    }
}

pub struct KeyedListUpdater<'a, C: Component, G, R> {
    list_context: KeyedListContext<'a>,
    render_context: KeyedListUpdaterContext<'a, C, G, R>,
}

pub struct KeyedListUpdaterContext<'a, C: Component, G, R> {
    comp: &'a Comp<C>,
    state: &'a C,
    fn_get_key: G,
    fn_render: R,
}

impl<'a, C, G, R> KeyedListUpdaterContext<'a, C, G, R>
where
    C: Component,
{
    pub fn new(comp: &'a Comp<C>, state: &'a C, fn_get_key: G, fn_render: R) -> Self {
        Self {
            comp,
            state,
            fn_get_key,
            fn_render,
        }
    }
    fn get_key<'k, I, K>(&self, entry_state: &'k I) -> &'k K
    where
        G: Fn(&I) -> &K,
    {
        (self.fn_get_key)(entry_state)
    }

    fn render<I>(&self, entry_state: I, r: NodesUpdater<C>)
    where
        R: Fn(I, NodesUpdater<C>),
    {
        (self.fn_render)(entry_state, r)
    }

    fn update_existing_entry<I>(
        &self,
        entry_state: I,
        parent: &web_sys::Node,
        old_entry: Option<(usize, &mut std::option::Option<KeyedEntry>)>,
        new_entry: Option<&mut std::option::Option<KeyedEntry>>,
        next_sibling: Option<&web_sys::Node>,
        relocating: bool,
    ) where
        R: Fn(I, NodesUpdater<C>),
    {
        let mut old_entry = old_entry.unwrap_throw().1.take();
        if relocating {
            old_entry
                .as_ref()
                .unwrap_throw()
                .group
                .insert_before_a_sibling(parent, next_sibling);
        }

        let nu = NodesUpdater::new(
            self.comp,
            self.state,
            ElementStatus::Existing,
            parent,
            next_sibling,
            old_entry.as_mut().unwrap_throw().group.nodes_mut(),
        );
        (self.fn_render)(entry_state, nu);
        *new_entry.expect_throw(
            "render::base::keyed_list::KeyedListUpdaterContext::update_existing_entry",
        ) = old_entry;
    }
}

impl<'a, C, G, R> KeyedListUpdater<'a, C, G, R>
where
    C: Component,
{
    pub fn new(
        list_context: KeyedListContext<'a>,
        render_context: KeyedListUpdaterContext<'a, C, G, R>,
    ) -> Self {
        Self {
            list_context,
            render_context,
        }
    }
    fn create_entry_for_new_entry(&self) -> (GroupedNodes, ElementStatus) {
        match &self.list_context.template {
            Some(template) => (template.group.clone_list_entry(), ElementStatus::JustCloned),
            None => (
                GroupedNodes::new("start of a list entry"),
                ElementStatus::JustCreated,
            ),
        }
    }

    pub fn update<I, K>(
        &mut self,
        entries_state_iter: impl Iterator<Item = I> + DoubleEndedIterator,
    ) -> super::RememberSettingSelectedOption
    where
        G: Fn(&I) -> &K,
        K: PartialEq<ListEntryKey>,
        R: Fn(I, NodesUpdater<C>),
        ListEntryKey: for<'k> From<&'k K>,
    {
        // No entries? Just clear the current list.
        if self.list_context.new_entry_count == 0 {
            self.remove_all_old_entries();
            return super::RememberSettingSelectedOption;
        }

        let mut entries_state_iter = entries_state_iter.peekable_double_ended();
        if self.list_context.require_init_template {
            // If the template is not available yet, it means that no entry has ever been rendered.
            // The current render is the first render to the list. Then we just take the first
            // entry off the list, render it, clone the rendered element. Put one element into
            // the list, store the other as a template

            let entry = self.render_an_entry(
                entries_state_iter
                    .next()
                    .expect_throw("Only non empty can reach here"),
            );
            if let Some(template) = self.list_context.template.as_mut() {
                template.rendered = true;
                template.group = entry.group.clone();
            }
            self.store_keyed_rendered_entry(entry);
        }
        loop {
            let mut count = self.update_same_key_entries_from_start(&mut entries_state_iter);
            count += self.update_same_key_entries_from_end(&mut entries_state_iter);
            count += self.update_moved_forward_entry(&mut entries_state_iter);
            count += self.update_moved_backward_entry(&mut entries_state_iter);
            if count == 0 {
                break;
            }
        }

        self.update_other_entries_in_middle(&mut entries_state_iter);
        super::RememberSettingSelectedOption
    }

    fn update_same_key_entries_from_start<I, K>(
        &mut self,
        entries_state_iter: &mut PeekableDoubleEndedIterator<impl Iterator<Item = I>>,
    ) -> usize
    where
        G: Fn(&I) -> &K,
        K: PartialEq<ListEntryKey>,
        R: Fn(I, NodesUpdater<C>),
        ListEntryKey: for<'k> From<&'k K>,
    {
        let mut count = 0;
        loop {
            match (entries_state_iter.peek(), self.list_context.old.peek()) {
                (Some(entry_state), Some(old_entry)) => {
                    let old_entry = old_entry
                        .1
                        .as_ref()
                        .expect_throw("render::base::keyed_list::KeyedListUpdater::update_same_key_entries_from_start");
                    if !self.render_context.get_key(entry_state).eq(&old_entry.key) {
                        return count;
                    }
                }
                _ => return count,
            }
            count += 1;
            self.render_context.update_existing_entry(
                entries_state_iter.next().unwrap_throw(),
                self.list_context.parent,
                self.list_context.old.next(),
                self.list_context.new.next(),
                self.list_context
                    .old
                    .peek()
                    .and_then(|old| old.1.as_ref().map(|entry| entry.group.flag_node())),
                false,
            );
        }
    }

    fn update_same_key_entries_from_end<I, K>(
        &mut self,
        entries_state_iter: &mut PeekableDoubleEndedIterator<
            impl Iterator<Item = I> + DoubleEndedIterator,
        >,
    ) -> usize
    where
        G: Fn(&I) -> &K,
        K: PartialEq<ListEntryKey>,
        R: Fn(I, NodesUpdater<C>),
        ListEntryKey: for<'k> From<&'k K>,
    {
        let mut count = 0;
        loop {
            let new_end_flag = match (
                entries_state_iter.peek_back(),
                self.list_context.old.peek_back(),
            ) {
                (Some(entry_state), Some(old_entry)) => {
                    // The old entry must have an entry in it
                    let old_entry = old_entry.1.as_ref().expect_throw(
                        "render::base::keyed_list::KeyedListUpdater::update_same_key_entries_from_end",
                    );

                    if !self.render_context.get_key(entry_state).eq(&old_entry.key) {
                        // The old entry and the new entry-state dont have the same key,
                        // so the entry-state is a new entry. Nothing more to do in this
                        // `update_same_key_entries_from_end` method.
                        return count;
                    }
                    // The keys are the same, so we need to continue the loop.
                    // The proccessed entry will be up by one from the bottom.
                    // And we will need to update `end_flag_for_the_next_entry_bottom_up`
                    // with this value
                    old_entry.group.flag_node().clone()
                }
                _ => return count,
            };
            count += 1;
            self.render_context.update_existing_entry(
                entries_state_iter.next_back().unwrap_throw(),
                self.list_context.parent,
                self.list_context.old.next_back(),
                self.list_context.new.next_back(),
                self.list_context
                    .end_flag_for_the_next_entry_bottom_up
                    .as_ref(),
                false,
            );
            self.list_context.end_flag_for_the_next_entry_bottom_up = Some(new_end_flag);
        }
    }

    fn update_moved_forward_entry<I, K>(
        &mut self,
        entries_state_iter: &mut PeekableDoubleEndedIterator<impl Iterator<Item = I>>,
    ) -> usize
    where
        G: Fn(&I) -> &K,
        K: PartialEq<ListEntryKey>,
        R: Fn(I, NodesUpdater<C>),
        ListEntryKey: for<'k> From<&'k K>,
    {
        match (entries_state_iter.peek(), self.list_context.old.peek_back()) {
            (Some(entry_state), Some(old_entry)) => {
                let old_entry = old_entry.1.as_ref().expect_throw(
                    "render::base::keyed_list::KeyedListUpdater::update_same_key_entries_from_end",
                );
                if !self.render_context.get_key(entry_state).eq(&old_entry.key) {
                    // No entry moved forward
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
            .and_then(|entry| entry.1.as_ref().map(|entry| entry.group.flag_node()));
        self.render_context.update_existing_entry(
            entries_state_iter.next().unwrap_throw(),
            self.list_context.parent,
            moved,
            self.list_context.new.next(),
            next_sibling,
            true,
        );
        1
    }

    fn update_moved_backward_entry<I, K>(
        &mut self,
        entries_state_iter: &mut PeekableDoubleEndedIterator<
            impl Iterator<Item = I> + DoubleEndedIterator,
        >,
    ) -> usize
    where
        G: Fn(&I) -> &K,
        K: PartialEq<ListEntryKey>,
        R: Fn(I, NodesUpdater<C>),
        ListEntryKey: for<'k> From<&'k K>,
    {
        let new_end_flag = match (entries_state_iter.peek_back(), self.list_context.old.peek()) {
            (Some(entry_state), Some(old_entry)) => {
                let old_entry = old_entry.1.as_ref().expect_throw(
                    "render::base::keyed_list::KeyedListUpdater::update_same_key_entries_from_end",
                );
                if !self.render_context.get_key(entry_state).eq(&old_entry.key) {
                    // No entry moved backward
                    return 0;
                }
                old_entry.group.flag_node().clone()
            }
            _ => return 0,
        };
        self.render_context.update_existing_entry(
            entries_state_iter.next_back().unwrap_throw(),
            self.list_context.parent,
            self.list_context.old.next(),
            self.list_context.new.next_back(),
            self.list_context
                .end_flag_for_the_next_entry_bottom_up
                .as_ref(),
            true,
        );
        self.list_context.end_flag_for_the_next_entry_bottom_up = Some(new_end_flag);
        1
    }

    fn update_other_entries_in_middle<I, K>(
        &mut self,
        entries_state_iter: &mut PeekableDoubleEndedIterator<impl Iterator<Item = I>>,
    ) where
        G: Fn(&I) -> &K,
        K: PartialEq<ListEntryKey>,
        R: Fn(I, NodesUpdater<C>),
        ListEntryKey: for<'k> From<&'k K>,
    {
        if entries_state_iter.peek().is_none() {
            self.remove_remain_entries();
            return;
        }

        if self.list_context.old.peek().is_none() {
            self.insert_remain_entries(entries_state_iter);
            return;
        }

        self.construct_old_entries_map_from_remaining_old_entries();

        // Using longest_increasing_subsequence find which elements should be moved around in the browser's DOM
        // and which should be stay still
        let mut entries_with_lis: Vec<_> = entries_state_iter
            .map(|entry| {
                let old_entry = self
                    .list_context
                    .old_entries_map
                    .remove(&self.render_context.get_key(&entry).into());
                EntryWithLis::new(entry, old_entry)
            })
            .collect();
        longest_increasing_subsequence(&mut entries_with_lis);

        self.remove_old_entries_that_still_in_old_entries_map();

        for iwl in entries_with_lis.into_iter().rev() {
            let EntryWithLis {
                entry_state,
                old_entry,
                lis,
            } = iwl;

            let (mut group, status) = match old_entry {
                Some(old_entry) => (old_entry.group, ElementStatus::Existing),
                None => self.create_entry_for_new_entry(),
            };

            let next_sibling = self
                .list_context
                .end_flag_for_the_next_entry_bottom_up
                .as_ref();
            if !lis {
                group.insert_before_a_sibling(self.list_context.parent, next_sibling);
            }

            let nu = NodesUpdater::new(
                self.render_context.comp,
                self.render_context.state,
                status,
                self.list_context.parent,
                next_sibling,
                group.nodes_mut(),
            );

            let key = self.render_context.get_key(&entry_state).into();
            self.render_context.render(entry_state, nu);

            self.list_context.end_flag_for_the_next_entry_bottom_up =
                Some(group.flag_node().clone());
            *self.list_context.new.next_back().expect_throw(
                "render::base::keyed_list::KeyedListUpdater::update_other_entries_in_the_middle",
            ) = Some(KeyedEntry::new(key, group));
        }
    }

    fn construct_old_entries_map_from_remaining_old_entries(&mut self) {
        self.list_context.old_entries_map.clear();
        for (index, entry) in self.list_context.old.by_ref() {
            let KeyedEntry { key, group } = entry.take().expect_throw(
                "render::base::keyed_list::KeyedListUpdater::construct_old_entries_map_from_remaining_old_entries",
            );
            self.list_context
                .old_entries_map
                .insert(key, OldEntry { index, group });
        }
    }

    fn remove_old_entries_that_still_in_old_entries_map(&mut self) {
        let parent = self.list_context.parent;
        self.list_context
            .old_entries_map
            .drain()
            .for_each(|(_, entry)| {
                entry.group.remove_from_dom(parent);
            })
    }

    fn remove_all_old_entries(&mut self) {
        self.list_context.parent.set_text_content(None);
        for (_, entry) in self.list_context.old.by_ref() {
            entry
                .take()
                .expect_throw("render::base::keyed_list::KeyedListUpdater::remove_all_old_entries");
        }
    }

    fn remove_remain_entries(&mut self) {
        let parent = self.list_context.parent;
        for (_, entry) in self.list_context.old.by_ref() {
            entry
                .take()
                .expect_throw("render::base::keyed_list::KeyedListUpdater::remove_remain_entries")
                .group
                .remove_from_dom(parent);
        }
    }

    fn insert_remain_entries<I, K>(
        &mut self,
        entries_state_iter: &mut PeekableDoubleEndedIterator<impl Iterator<Item = I>>,
    ) where
        G: Fn(&I) -> &K,
        K: PartialEq<ListEntryKey>,
        R: Fn(I, NodesUpdater<C>),
        ListEntryKey: for<'k> From<&'k K>,
    {
        for entry_state in entries_state_iter {
            let ke = self.render_an_entry(entry_state);
            self.store_keyed_rendered_entry(ke);
        }
    }

    fn render_an_entry<I, K>(&mut self, entry_state: I) -> KeyedEntry
    where
        G: Fn(&I) -> &K,
        K: PartialEq<ListEntryKey>,
        R: Fn(I, NodesUpdater<C>),
        ListEntryKey: for<'k> From<&'k K>,
    {
        let (mut group, status) = self.create_entry_for_new_entry();

        let next_sibling = self
            .list_context
            .end_flag_for_the_next_entry_bottom_up
            .as_ref();
        group.insert_before_a_sibling(self.list_context.parent, next_sibling);

        let nu = NodesUpdater::new(
            self.render_context.comp,
            self.render_context.state,
            status,
            self.list_context.parent,
            next_sibling,
            group.nodes_mut(),
        );

        let key = self.render_context.get_key(&entry_state).into();
        self.render_context.render(entry_state, nu);

        KeyedEntry::new(key, group)
    }

    fn store_keyed_rendered_entry(&mut self, ke: KeyedEntry) {
        *self
            .list_context
            .new
            .next()
            .expect_throw("render::base::keyed_list::KeyedListUpdater::inser_remain_entries") =
            Some(ke);
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

#[derive(Debug)]
struct EntryWithLis<I> {
    entry_state: I,
    old_entry: Option<OldEntry>,
    lis: bool,
}

impl<I> EntryWithLis<I> {
    fn new(entry_state: I, old_entry: Option<OldEntry>) -> Self {
        Self {
            entry_state,
            old_entry,
            lis: false,
        }
    }
}

// Copied from https://github.com/axelf4/lis and modified to work with Spair.
fn longest_increasing_subsequence<I>(entries: &mut [EntryWithLis<I>]) {
    let mut p = vec![0; entries.len()];
    // indices of the new entries
    let mut m = Vec::with_capacity(entries.len());
    // only iter through entries with old index
    let mut it = entries
        .iter()
        .enumerate()
        .filter(|(_, x)| x.old_entry.is_some());
    if let Some((i, _)) = it.next() {
        m.push(i);
    } else {
        return;
    }

    for (i, x) in it {
        // Test whether a[i] can extend the current sequence
        if entries[*m.last().unwrap_throw()]
            .old_entry
            .as_ref()
            .unwrap_throw()
            .index
            .cmp(&x.old_entry.as_ref().unwrap_throw().index)
            == std::cmp::Ordering::Less
        {
            p[i] = *m.last().unwrap_throw();
            m.push(i);
            continue;
        }

        // Binary search for largest j ≤ m.len() such that a[m[j]] < a[i]
        let j = match m.binary_search_by(|&j| {
            entries[j]
                .old_entry
                .as_ref()
                .unwrap_throw()
                .index
                .cmp(&x.old_entry.as_ref().unwrap_throw().index)
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

#[cfg(test)]
mod keyed_list_with_render_tests {
    use wasm_bindgen_test::*;

    use crate::dom::{GroupedNodes, Node};

    impl super::EntryWithLis<&()> {
        fn index(index: usize) -> Self {
            Self {
                entry_state: &(),
                old_entry: Some(super::OldEntry {
                    index,
                    group: GroupedNodes::new("start of list entry"),
                }),
                lis: false,
            }
        }
        fn none() -> Self {
            Self {
                entry_state: &(),
                old_entry: None,
                lis: false,
            }
        }
    }

    fn collect_lis(mut entries: Vec<super::EntryWithLis<&()>>) -> Vec<usize> {
        super::longest_increasing_subsequence(&mut entries[..]);
        entries
            .iter()
            .flat_map(|entry| {
                if entry.lis {
                    entry.old_entry.as_ref().map(|old_entry| old_entry.index)
                } else {
                    None
                }
            })
            .collect()
    }

    fn collect_lis_from_index(indices: &[usize]) -> Vec<usize> {
        let entries = indices
            .iter()
            .map(|i| super::EntryWithLis::index(*i))
            .collect();
        collect_lis(entries)
    }

    #[wasm_bindgen_test]
    fn lis_with_none() {
        let entries = vec![
            super::EntryWithLis::index(5),
            super::EntryWithLis::index(1),
            super::EntryWithLis::index(3),
            super::EntryWithLis::none(),
            super::EntryWithLis::index(6),
            super::EntryWithLis::index(8),
            super::EntryWithLis::none(),
            super::EntryWithLis::index(9),
            super::EntryWithLis::index(0),
            super::EntryWithLis::index(7),
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

    macro_rules! make_keyed_list_test {
        ($mode:expr) => {
            make_a_test_component! {
                type: Vec<&'static str>;
                init: Vec::new();
                render_fn: fn render(&self, element: crate::Element<Self>) {
                    element.keyed_list(self.0.iter(), $mode, |entry| *entry, render_str);
                }
            }

            fn render_str(value: &&str, nodes: crate::Nodes<TestComponent>) {
                nodes.update_text(*value);
            }

            fn collect_from_keyed_list(nodes: &[crate::dom::Node]) -> Vec<String> {
                if let Node::KeyedList(kl) = nodes.first().unwrap_throw() {
                    kl.active_nodes()
                        .iter()
                        .map(|entry| {
                            entry
                                .as_ref()
                                .unwrap_throw()
                                .group
                                .nodes()
                                .nodes_vec()
                                .first()
                                .unwrap_throw()
                        })
                        .map(|entry| match entry {
                            Node::Text(text) => text.test_string(),
                            _ => panic!("Should be a text?"),
                        })
                        .collect()
                } else {
                    Vec::new()
                }
            }

            let test = Test::set_up();

            let empty: Vec<&'static str> = Vec::new();
            test.update(empty.clone());
            assert_eq!(Some(""), test.text_content().as_deref());
            assert_eq!(empty, test.execute_on_nodes(collect_from_keyed_list));

            let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"];
            test.update(data.clone());
            assert_eq!(data, test.execute_on_nodes(collect_from_keyed_list));
            assert_eq!(Some("abcdefghijk"), test.text_content().as_deref());

            // Random shuffle + addition
            let data = vec!["f", "b", "d", "l", "g", "i", "m", "j", "a", "h", "k"];
            test.update(data.clone());
            assert_eq!(Some("fbdlgimjahk"), test.text_content().as_deref());
            assert_eq!(data, test.execute_on_nodes(collect_from_keyed_list));

            // Empty the list
            test.update(empty.clone());
            assert_eq!(Some(""), test.text_content().as_deref());
            assert_eq!(empty, test.execute_on_nodes(collect_from_keyed_list));

            // Add back
            let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"];
            test.update(data.clone());
            assert_eq!(data, test.execute_on_nodes(collect_from_keyed_list));
            assert_eq!(Some("abcdefghijk"), test.text_content().as_deref());

            // Forward
            let data = vec!["a", "i", "b", "c", "d", "e", "f", "g", "h", "j", "k"];
            test.update(data.clone());
            assert_eq!(data, test.execute_on_nodes(collect_from_keyed_list));
            assert_eq!(Some("aibcdefghjk"), test.text_content().as_deref());

            // Backward
            let data = vec!["a", "i", "c", "d", "e", "f", "g", "h", "b", "j", "k"];
            test.update(data.clone());
            assert_eq!(data, test.execute_on_nodes(collect_from_keyed_list));
            assert_eq!(Some("aicdefghbjk"), test.text_content().as_deref());

            // Swap
            let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"];
            test.update(data.clone());
            assert_eq!(data, test.execute_on_nodes(collect_from_keyed_list));
            assert_eq!(Some("abcdefghijk"), test.text_content().as_deref());

            // Remove middle
            let data = vec!["a", "b", "c", "d", "i", "j", "k"];
            test.update(data.clone());
            assert_eq!(data, test.execute_on_nodes(collect_from_keyed_list));
            assert_eq!(Some("abcdijk"), test.text_content().as_deref());

            // Insert middle
            let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"];
            test.update(data.clone());
            assert_eq!(data, test.execute_on_nodes(collect_from_keyed_list));
            assert_eq!(Some("abcdefghijk"), test.text_content().as_deref());

            // Remove start
            let data = vec!["d", "e", "f", "g", "h", "i", "j", "k"];
            test.update(data.clone());
            assert_eq!(data, test.execute_on_nodes(collect_from_keyed_list));
            assert_eq!(Some("defghijk"), test.text_content().as_deref());

            // Insert start
            let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"];
            test.update(data.clone());
            assert_eq!(data, test.execute_on_nodes(collect_from_keyed_list));
            assert_eq!(Some("abcdefghijk"), test.text_content().as_deref());

            // Remove end
            let data = vec!["a", "b", "c", "d", "e", "f", "g", "h"];
            test.update(data.clone());
            assert_eq!(data, test.execute_on_nodes(collect_from_keyed_list));
            assert_eq!(Some("abcdefgh"), test.text_content().as_deref());

            // Append end
            let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"];
            test.update(data.clone());
            assert_eq!(data, test.execute_on_nodes(collect_from_keyed_list));
            assert_eq!(Some("abcdefghijk"), test.text_content().as_deref());
        };
    }

    #[wasm_bindgen_test::wasm_bindgen_test]
    fn keyed_list_clone() {
        make_keyed_list_test!(crate::ListElementCreation::Clone);
    }

    #[wasm_bindgen_test::wasm_bindgen_test]
    fn keyed_list_new() {
        make_keyed_list_test!(crate::ListElementCreation::New);
    }
}
