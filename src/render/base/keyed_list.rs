use wasm_bindgen::{JsCast, UnwrapThrowExt};

use super::ElementUpdater;
use crate::{
    component::{Comp, Component},
    dom::{
        AChildNode, Element, ElementStatus, ElementTag, KeyedElement, KeyedList, ListItemKey,
        ListItemTemplate, OldElement,
    },
};

pub struct KeyedListContext<'a, E> {
    parent: &'a web_sys::Node,
    root_item_tag: E,
    old: PeekableDoubleEndedIterator<
        std::iter::Enumerate<std::slice::IterMut<'a, Option<KeyedElement>>>,
    >,
    new: PeekableDoubleEndedIterator<std::slice::IterMut<'a, Option<KeyedElement>>>,
    old_elements_map: &'a mut std::collections::HashMap<ListItemKey, OldElement>,
    new_item_count: usize,
    next_sibling: Option<web_sys::Element>,
    template: Option<&'a mut ListItemTemplate>,
    require_init_template: bool,
}

impl<'a, E: ElementTag> KeyedListContext<'a, E> {
    pub fn new(
        list: &'a mut KeyedList,
        root_item_tag: E,
        new_item_count: usize,
        parent: &'a web_sys::Node,
        use_template: bool,
    ) -> Self {
        list.pre_update(new_item_count);

        let require_init_template = match use_template {
            true => list.require_init_template(|| Element::new_ns(root_item_tag)),
            false => false,
        };

        let (template, old, new, old_elements_map) = list.items_mut();
        KeyedListContext {
            parent,
            root_item_tag,
            old: old.iter_mut().enumerate().peekable_double_ended(),
            new: new.iter_mut().peekable_double_ended(),
            old_elements_map,
            new_item_count,
            next_sibling: None,
            template,
            require_init_template,
        }
    }
}

pub struct KeyedListUpdater<'a, C: Component, E, G, R> {
    list_context: KeyedListContext<'a, E>,
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
    fn get_key<'k, I, K>(&self, item_state: &'k I) -> &'k K
    where
        G: Fn(&I) -> &K,
    {
        (self.fn_get_key)(item_state)
    }

    fn render<I>(&self, item_state: I, r: ElementUpdater<C>)
    where
        R: Fn(I, ElementUpdater<C>),
    {
        (self.fn_render)(item_state, r)
    }

    fn update_existing_item<I>(
        &self,
        item_state: I,
        old_item: Option<(usize, &mut std::option::Option<KeyedElement>)>,
        new_item: Option<&mut std::option::Option<KeyedElement>>,
        next_sibling: Option<&web_sys::Element>,
        fn_insert: impl FnOnce(&Element, Option<&web_sys::Element>),
    ) where
        R: Fn(I, ElementUpdater<C>),
    {
        let mut old_item = old_item.unwrap_throw().1.take();
        fn_insert(&old_item.as_ref().unwrap_throw().element, next_sibling);

        let er = ElementUpdater::new(
            self.comp,
            self.state,
            &mut old_item.as_mut().unwrap_throw().element,
            ElementStatus::Existing,
        );
        (self.fn_render)(item_state, er);
        *new_item.expect_throw(
            "render::base::keyed_list::KeyedListUpdaterContext::update_existing_item",
        ) = old_item;
    }
}

impl<'a, C, E, G, R> KeyedListUpdater<'a, C, E, G, R>
where
    C: Component,
    E: ElementTag,
{
    pub fn new(
        list_context: KeyedListContext<'a, E>,
        render_context: KeyedListUpdaterContext<'a, C, G, R>,
    ) -> Self {
        Self {
            list_context,
            render_context,
        }
    }
    fn create_element_for_new_item(&self) -> (Element, ElementStatus) {
        match &self.list_context.template {
            Some(template) => (Clone::clone(&template.element), ElementStatus::JustCloned),
            None => (
                Element::new_ns(self.list_context.root_item_tag),
                ElementStatus::JustCreated,
            ),
        }
    }

    pub fn update<I, K>(
        &mut self,
        items_state_iter: impl Iterator<Item = I> + DoubleEndedIterator,
    ) -> super::RememberSettingSelectedOption
    where
        G: Fn(&I) -> &K,
        K: PartialEq<ListItemKey>,
        R: Fn(I, ElementUpdater<C>),
        ListItemKey: for<'k> From<&'k K>,
    {
        // No items? Just clear the current list.
        if self.list_context.new_item_count == 0 {
            self.remove_all_old_items();
            return super::RememberSettingSelectedOption;
        }

        let mut items_state_iter = items_state_iter.peekable_double_ended();
        if self.list_context.require_init_template {
            // If the template is not available yet, it means that no item has ever been rendered.
            // The current render is the first render to the list. Then we just take the first
            // item off the list, render it, clone the rendered element. Put one element into
            // the list, store the other as a template

            let ke = self.render_an_item(
                items_state_iter
                    .next()
                    .expect_throw("Only non empty can reach here"),
            );
            if let Some(template) = self.list_context.template.as_mut() {
                template.rendered = true;
                template.element = ke.element.clone();
            }
            self.store_keyed_rendered_item(ke);
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

    fn update_same_key_items_from_start<I, K>(
        &mut self,
        items_state_iter: &mut PeekableDoubleEndedIterator<impl Iterator<Item = I>>,
    ) -> usize
    where
        G: Fn(&I) -> &K,
        K: PartialEq<ListItemKey>,
        R: Fn(I, ElementUpdater<C>),
        ListItemKey: for<'k> From<&'k K>,
    {
        let mut count = 0;
        loop {
            match (items_state_iter.peek(), self.list_context.old.peek()) {
                (Some(item_state), Some(item)) => {
                    let item = item
                        .1
                        .as_ref()
                        .expect_throw("render::base::keyed_list::KeyedListUpdater::update_same_key_items_from_start");
                    if !self.render_context.get_key(item_state).eq(&item.key) {
                        return count;
                    }
                }
                _ => return count,
            }
            count += 1;
            self.render_context.update_existing_item(
                items_state_iter.next().unwrap_throw(),
                self.list_context.old.next(),
                self.list_context.new.next(),
                None,
                |_, _| {},
            );
        }
    }

    fn update_same_key_items_from_end<I, K>(
        &mut self,
        items_state_iter: &mut PeekableDoubleEndedIterator<
            impl Iterator<Item = I> + DoubleEndedIterator,
        >,
    ) -> usize
    where
        G: Fn(&I) -> &K,
        K: PartialEq<ListItemKey>,
        R: Fn(I, ElementUpdater<C>),
        ListItemKey: for<'k> From<&'k K>,
    {
        let mut count = 0;
        loop {
            let ws_element = match (
                items_state_iter.peek_back(),
                self.list_context.old.peek_back(),
            ) {
                (Some(item_state), Some(item)) => {
                    let item = item.1.as_ref().expect_throw(
                        "render::base::keyed_list::KeyedListUpdater::update_same_key_items_from_end",
                    );

                    if !self.render_context.get_key(item_state).eq(&item.key) {
                        return count;
                    }
                    item.element.ws_element().clone()
                }
                _ => return count,
            };
            count += 1;
            self.render_context.update_existing_item(
                items_state_iter.next_back().unwrap_throw(),
                self.list_context.old.next_back(),
                self.list_context.new.next_back(),
                None,
                |_, _| {},
            );
            self.list_context.next_sibling = Some(ws_element.into_inner());
        }
    }

    fn update_moved_forward_item<I, K>(
        &mut self,
        items_state_iter: &mut PeekableDoubleEndedIterator<impl Iterator<Item = I>>,
    ) -> usize
    where
        G: Fn(&I) -> &K,
        K: PartialEq<ListItemKey>,
        R: Fn(I, ElementUpdater<C>),
        ListItemKey: for<'k> From<&'k K>,
    {
        match (items_state_iter.peek(), self.list_context.old.peek_back()) {
            (Some(item_state), Some(item)) => {
                let item = item.1.as_ref().expect_throw(
                    "render::base::keyed_list::KeyedListUpdater::update_same_key_items_from_end",
                );
                if !self.render_context.get_key(item_state).eq(&item.key) {
                    return 0;
                }
            }
            _ => return 0,
        }
        let moved = self.list_context.old.next_back();
        let next_sibling = self.list_context.old.peek().and_then(|item| {
            item.1
                .as_ref()
                .map(|item| item.element.ws_element().as_ref())
        });
        let parent = self.list_context.parent;
        self.render_context.update_existing_item(
            items_state_iter.next().unwrap_throw(),
            moved,
            self.list_context.new.next(),
            next_sibling,
            |element, next_sibling| {
                element.insert_before_a_sibling(
                    parent,
                    next_sibling.map(|element| element.unchecked_ref()),
                );
            },
        );
        1
    }

    fn update_moved_backward_item<I, K>(
        &mut self,
        items_state_iter: &mut PeekableDoubleEndedIterator<
            impl Iterator<Item = I> + DoubleEndedIterator,
        >,
    ) -> usize
    where
        G: Fn(&I) -> &K,
        K: PartialEq<ListItemKey>,
        R: Fn(I, ElementUpdater<C>),
        ListItemKey: for<'k> From<&'k K>,
    {
        let new_next_sibling = match (items_state_iter.peek_back(), self.list_context.old.peek()) {
            (Some(item_state), Some(item)) => {
                let item = item.1.as_ref().expect_throw(
                    "render::base::keyed_list::KeyedListUpdater::update_same_key_items_from_end",
                );
                if !self.render_context.get_key(item_state).eq(&item.key) {
                    return 0;
                }
                item.element.ws_element().clone()
            }
            _ => return 0,
        };
        self.render_context.update_existing_item(
            items_state_iter.next_back().unwrap_throw(),
            self.list_context.old.next(),
            self.list_context.new.next_back(),
            self.list_context.next_sibling.as_ref(),
            |element, next_sibling| {
                element.insert_before_a_sibling(
                    self.list_context.parent,
                    next_sibling.map(|element| element.unchecked_ref()),
                );
            },
        );
        self.list_context.next_sibling = Some(new_next_sibling.into_inner());
        1
    }

    fn update_other_items_in_middle<I, K>(
        &mut self,
        items_state_iter: &mut PeekableDoubleEndedIterator<impl Iterator<Item = I>>,
    ) where
        G: Fn(&I) -> &K,
        K: PartialEq<ListItemKey>,
        R: Fn(I, ElementUpdater<C>),
        ListItemKey: for<'k> From<&'k K>,
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
                    .remove(&self.render_context.get_key(&item).into());
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
                Some(old_element) => (old_element.element, ElementStatus::Existing),
                None => self.create_element_for_new_item(),
            };

            let er = ElementUpdater::new(
                self.render_context.comp,
                self.render_context.state,
                &mut element,
                status,
            );

            let key = self.render_context.get_key(&item_state).into();
            self.render_context.render(item_state, er);
            if !lis {
                let next_sibling = self
                    .list_context
                    .next_sibling
                    .as_ref()
                    .map(|element| element.unchecked_ref());
                element.insert_before_a_sibling(self.list_context.parent, next_sibling);
            }

            self.list_context.next_sibling = Some(element.ws_element().clone().into_inner());
            *self.list_context.new.next_back().expect_throw(
                "render::base::keyed_list::KeyedListUpdater::update_other_items_in_the_middle",
            ) = Some(KeyedElement::new(key, element));
        }
    }

    fn construct_old_elements_map_from_remaining_old_elements(&mut self) {
        self.list_context.old_elements_map.clear();
        for (index, item) in self.list_context.old.by_ref() {
            //while let Some((index, item)) = self.list_context.old.next() {
            let KeyedElement { key, element } = item.take().expect_throw(
                "render::base::keyed_list::KeyedListUpdater::construct_old_elements_map_from_remaining_old_elements",
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
        for (_, item) in self.list_context.old.by_ref() {
            // while let Some((_, item)) = self.list_context.old.next() {
            item.take()
                .expect_throw("render::base::keyed_list::KeyedListUpdater::remove_all_old_items");
        }
    }

    fn remove_remain_items(&mut self) {
        let parent = self.list_context.parent;
        for (_, item) in self.list_context.old.by_ref() {
            //while let Some((_, item)) = self.list_context.old.next() {
            item.take()
                .expect_throw("render::base::keyed_list::KeyedListUpdater::remove_remain_items")
                .element
                .remove_from(parent);
        }
    }

    fn insert_remain_items<I, K>(
        &mut self,
        items_state_iter: &mut PeekableDoubleEndedIterator<impl Iterator<Item = I>>,
    ) where
        G: Fn(&I) -> &K,
        K: PartialEq<ListItemKey>,
        R: Fn(I, ElementUpdater<C>),
        ListItemKey: for<'k> From<&'k K>,
    {
        for item_state in items_state_iter {
            let ke = self.render_an_item(item_state);
            self.store_keyed_rendered_item(ke);
        }
    }

    fn render_an_item<I, K>(&mut self, item_state: I) -> KeyedElement
    where
        G: Fn(&I) -> &K,
        K: PartialEq<ListItemKey>,
        R: Fn(I, ElementUpdater<C>),
        ListItemKey: for<'k> From<&'k K>,
    {
        let (mut element, status) = self.create_element_for_new_item();

        let er = ElementUpdater::new(
            self.render_context.comp,
            self.render_context.state,
            &mut element,
            status,
        );

        let key = self.render_context.get_key(&item_state).into();
        self.render_context.render(item_state, er);
        element.insert_before_a_sibling(
            self.list_context.parent,
            self.list_context
                .next_sibling
                .as_ref()
                .map(|element| element.unchecked_ref()),
        );
        KeyedElement::new(key, element)
    }

    fn store_keyed_rendered_item(&mut self, ke: KeyedElement) {
        *self
            .list_context
            .new
            .next()
            .expect_throw("render::base::keyed_list::KeyedListUpdater::inser_remain_items") =
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
struct ItemWithLis<I> {
    item_state: I,
    old_element: Option<OldElement>,
    lis: bool,
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
mod keyed_list_with_render_tests {
    use wasm_bindgen_test::*;

    use crate::dom::{Element, Keyed, Node};
    use crate::render::html::{ElementRender, HtmlTag};

    impl super::ItemWithLis<&()> {
        fn index(index: usize) -> Self {
            Self {
                item_state: &(),
                old_element: Some(super::OldElement {
                    index,
                    element: Element::new_ns(HtmlTag("div")),
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

    impl Keyed for &&'static str {
        type Key = &'static str;
        fn key(&self) -> &Self::Key {
            self
        }
    }

    macro_rules! make_keyed_list_test {
        ($mode:expr) => {
            make_a_test_component! {
                type: Vec<&'static str>;
                init: Vec::new();
                render_fn: fn render(&self, element: crate::Element<Self>) {
                    element.keyed_list(self.0.iter(), $mode);
                }
            }

            impl ElementRender<TestComponent> for &&str {
                const ELEMENT_TAG: &'static str = "span";
                fn render(self, item: crate::Element<TestComponent>) {
                    item.rupdate(*self);
                }
            }

            fn collect_from_keyed_list(nodes: &[crate::dom::Node]) -> Vec<String> {
                if let Node::KeyedList(kl) = nodes.first().unwrap_throw() {
                    kl.active_nodes()
                        .iter()
                        .map(|item| {
                            item.as_ref()
                                .unwrap_throw()
                                .element
                                .nodes()
                                .nodes_vec()
                                .first()
                                .unwrap_throw()
                        })
                        .map(|item| match item {
                            Node::Text(text) => text.text().to_string(),
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
