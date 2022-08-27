use wasm_bindgen::{JsCast, UnwrapThrowExt};

use super::ElementRender;
use crate::{
    component::{Comp, Component},
    dom::{AChildNode, Element, ElementStatus, Key, KeyedElement, KeyedList, OldElement},
};

pub struct KeyedListContext<'a> {
    parent: &'a web_sys::Node,
    root_item_tag: &'a str,
    old: PeekableDoubleEndedIterator<
        std::iter::Enumerate<std::slice::IterMut<'a, Option<KeyedElement>>>,
    >,
    new: PeekableDoubleEndedIterator<std::slice::IterMut<'a, Option<KeyedElement>>>,
    old_elements_map: &'a mut std::collections::HashMap<Key, OldElement>,
    new_item_count: usize,
    next_sibling: Option<web_sys::Element>,
    name_space: &'a str,
    template: Option<&'a mut Element>,
    require_init_template: bool,
}

impl<'a> KeyedListContext<'a> {
    pub fn new(
        list: &'a mut KeyedList,
        root_item_tag: &'a str,
        name_space: &'a str,
        new_item_count: usize,
        parent: &'a web_sys::Node,
        use_template: bool,
    ) -> KeyedListContext<'a> {
        list.pre_render(new_item_count);

        let mut require_init_template = false;
        if use_template {
            require_init_template =
                list.set_template(|| Element::new_ns(name_space, root_item_tag));
        }
        let (template, old, new, old_elements_map) = list.items_mut();
        KeyedListContext {
            parent,
            root_item_tag,
            old: old.iter_mut().enumerate().peekable_double_ended(),
            new: new.iter_mut().peekable_double_ended(),
            old_elements_map,
            new_item_count,
            next_sibling: None,
            name_space,
            template,
            require_init_template,
        }
    }
}

pub struct KeyedListRender<'a, C: Component, G, R> {
    list_context: KeyedListContext<'a>,
    render_context: RenderContext<'a, C, G, R>,
}

pub struct RenderContext<'a, C: Component, G, R> {
    comp: &'a Comp<C>,
    state: &'a C,
    fn_get_key: G,
    fn_render: R,
}

impl<'a, C, G, R> RenderContext<'a, C, G, R>
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
    fn get_key<I, K>(&self, item_state: I) -> K
    where
        G: Fn(I) -> K,
    {
        (self.fn_get_key)(item_state)
    }

    fn render<I>(&self, item_state: I, r: ElementRender<C>)
    where
        for<'r> R: Fn(I, ElementRender<'r, C>),
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
        for<'r> R: Fn(I, ElementRender<'r, C>),
    {
        let mut old_item = old_item.unwrap_throw().1.take();
        fn_insert(&old_item.as_ref().unwrap_throw().element, next_sibling);

        let er = ElementRender::new(
            self.comp,
            self.state,
            &mut old_item.as_mut().unwrap_throw().element,
            ElementStatus::Existing,
        );
        (self.fn_render)(item_state, er);
        *new_item.expect_throw("render::base::keyed_list::RenderContext::update_existing_item") =
            old_item;
    }
}

impl<'a, C, G, R> KeyedListRender<'a, C, G, R>
where
    C: Component,
{
    pub fn new(
        list_context: KeyedListContext<'a>,
        render_context: RenderContext<'a, C, G, R>,
    ) -> Self {
        Self {
            list_context,
            render_context,
        }
    }
    fn create_element_for_new_item(&self) -> (Element, ElementStatus) {
        match &self.list_context.template {
            Some(template) => (Clone::clone(*template), ElementStatus::JustCloned),
            None => (
                Element::new_ns(
                    self.list_context.name_space,
                    self.list_context.root_item_tag,
                ),
                ElementStatus::JustCreated,
            ),
        }
    }

    pub fn update<I, K>(
        &mut self,
        items_state_iter: impl Iterator<Item = I> + DoubleEndedIterator,
    ) -> super::RememberSettingSelectedOption
    where
        I: Copy,
        G: Fn(I) -> K,
        K: Into<Key> + PartialEq<Key>,
        for<'er> R: Fn(I, ElementRender<'er, C>),
    {
        // No items? Just clear the current list.
        if self.list_context.new_item_count == 0 {
            self.remove_all_old_items();
            return super::RememberSettingSelectedOption;
        }

        let mut items_state_iter = items_state_iter.peekable_double_ended();
        if self.list_context.require_init_template {
            let er = ElementRender::new(
                self.render_context.comp,
                self.render_context.state,
                self.list_context.template.as_mut().unwrap(),
                ElementStatus::JustCreated,
            );

            // Render the template with the first item's state
            self.render_context
                .render(*items_state_iter.peek().unwrap_throw(), er);
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
        I: Copy,
        G: Fn(I) -> K,
        K: Into<Key> + PartialEq<Key>,
        for<'er> R: Fn(I, ElementRender<'er, C>),
    {
        let mut count = 0;
        loop {
            match (items_state_iter.peek(), self.list_context.old.peek()) {
                (Some(item_state), Some(item)) => {
                    let item = item
                        .1
                        .as_ref()
                        .expect_throw("render::base::keyed_list::KeyedListRender::update_same_key_items_from_start");
                    if !self.render_context.get_key(*item_state).eq(&item.key) {
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
        I: Copy,
        G: Fn(I) -> K,
        K: Into<Key> + PartialEq<Key>,
        for<'er> R: Fn(I, ElementRender<'er, C>),
    {
        let mut count = 0;
        loop {
            let ws_element = match (
                items_state_iter.peek_back(),
                self.list_context.old.peek_back(),
            ) {
                (Some(item_state), Some(item)) => {
                    let item = item.1.as_ref().expect_throw(
                        "render::base::keyed_list::KeyedListRender::update_same_key_items_from_end",
                    );

                    if !self.render_context.get_key(*item_state).eq(&item.key) {
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
        I: Copy,
        G: Fn(I) -> K,
        K: Into<Key> + PartialEq<Key>,
        for<'er> R: Fn(I, ElementRender<'er, C>),
    {
        match (items_state_iter.peek(), self.list_context.old.peek_back()) {
            (Some(item_state), Some(item)) => {
                let item = item.1.as_ref().expect_throw(
                    "render::base::keyed_list::KeyedListRender::update_same_key_items_from_end",
                );
                if !self.render_context.get_key(*item_state).eq(&item.key) {
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
        I: Copy,
        G: Fn(I) -> K,
        K: Into<Key> + PartialEq<Key>,
        for<'er> R: Fn(I, ElementRender<'er, C>),
    {
        let new_next_sibling = match (items_state_iter.peek_back(), self.list_context.old.peek()) {
            (Some(item_state), Some(item)) => {
                let item = item.1.as_ref().expect_throw(
                    "render::base::keyed_list::KeyedListRender::update_same_key_items_from_end",
                );
                if !self.render_context.get_key(*item_state).eq(&item.key) {
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
        I: Copy,
        G: Fn(I) -> K,
        K: Into<Key> + PartialEq<Key>,
        for<'er> R: Fn(I, ElementRender<'er, C>),
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
                    .remove(&self.render_context.get_key(item).into());
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

            let er = ElementRender::new(
                self.render_context.comp,
                self.render_context.state,
                &mut element,
                status,
            );

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
                "render::base::keyed_list::KeyedListRender::update_other_items_in_the_middle",
            ) = Some(KeyedElement::new(
                self.render_context.get_key(item_state).into(),
                element,
            ));
        }
    }

    fn construct_old_elements_map_from_remaining_old_elements(&mut self) {
        self.list_context.old_elements_map.clear();
        while let Some((index, item)) = self.list_context.old.next() {
            let KeyedElement { key, element } = item.take().expect_throw(
                "render::base::keyed_list::KeyedListRender::construct_old_elements_map_from_remaining_old_elements",
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
                .expect_throw("render::base::keyed_list::KeyedListRender::remove_all_old_items");
        }
    }

    fn remove_remain_items(&mut self) {
        let parent = self.list_context.parent;
        while let Some((_, item)) = self.list_context.old.next() {
            item.take()
                .expect_throw("render::base::keyed_list::KeyedListRender::remove_remain_items")
                .element
                .remove_from(parent);
        }
    }

    fn insert_remain_items<I, K>(
        &mut self,
        items_state_iter: &mut PeekableDoubleEndedIterator<impl Iterator<Item = I>>,
    ) where
        I: Copy,
        G: Fn(I) -> K,
        K: Into<Key> + PartialEq<Key>,
        for<'er> R: Fn(I, ElementRender<'er, C>),
    {
        for item_state in items_state_iter {
            let (mut element, status) = self.create_element_for_new_item();

            let er = ElementRender::new(
                self.render_context.comp,
                self.render_context.state,
                &mut element,
                status,
            );

            self.render_context.render(item_state, er);
            element.insert_before_a_sibling(
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
                .expect_throw("render::base::keyed_list::KeyedListRender::inser_remain_items") =
                Some(KeyedElement::new(
                    self.render_context.get_key(item_state).into(),
                    element,
                ));
        }
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
    use wasm_bindgen::UnwrapThrowExt;
    use wasm_bindgen_test::*;

    use crate::dom::{Element, Node};
    use crate::render::ListElementCreation;
    use crate::render::{
        base::ElementRender,
        html::{HemsForKeyedList, HtmlElementRender, HtmlTag},
    };

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
            let root = crate::dom::Element::new_ns(HtmlTag("div"));
            let _rc =
                crate::component::RcComp::with_ws_root(root.ws_element().clone().into_inner());
            _rc.set_state(Unit);

            let comp = _rc.comp();
            Self { root, _rc, comp }
        }

        fn create_render(&mut self) -> HtmlElementRender<Unit> {
            ElementRender::new(
                &self.comp,
                &Unit,
                &mut self.root,
                crate::dom::ElementStatus::Existing,
            )
            .into()
        }

        fn collect_from_keyed_list(&self) -> Vec<String> {
            if let Node::KeyedList(kl) = self.root.nodes().nodes_vec().first().unwrap_throw() {
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
    }

    fn render(item: &&str, span: crate::Element<Unit>) {
        use crate::render::html::MethodsForHtmlElementContent;
        span.update_render(*item);
    }

    fn get_key<'a>(item: &'a &str) -> &'a str {
        *item
    }

    #[wasm_bindgen_test]
    fn keyed_list_with_template() {
        keyed_list(ListElementCreation::Clone);
    }

    #[wasm_bindgen_test]
    fn keyed_list_no_template() {
        keyed_list(ListElementCreation::New);
    }

    fn keyed_list(mode: ListElementCreation) {
        let mut pa = PhantomApp::new();

        let empty: Vec<&'static str> = Vec::new();
        let _ = pa
            .create_render()
            .keyed_list_with_render(&empty, mode, "span", get_key, render);
        assert_eq!(
            Some(""),
            pa.root.ws_element().as_ref().text_content().as_deref()
        );
        assert_eq!(empty, pa.collect_from_keyed_list());

        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"];
        let _ = pa
            .create_render()
            .keyed_list_with_render(&data, mode, "span", get_key, render);
        assert_eq!(data, pa.collect_from_keyed_list());
        assert_eq!(
            Some("abcdefghijk"),
            pa.root.ws_element().as_ref().text_content().as_deref()
        );

        // Random shuffle + addition
        let data = vec!["f", "b", "d", "l", "g", "i", "m", "j", "a", "h", "k"];
        let _ = pa
            .create_render()
            .keyed_list_with_render(&data, mode, "span", get_key, render);
        assert_eq!(
            Some("fbdlgimjahk"),
            pa.root.ws_element().as_ref().text_content().as_deref()
        );
        assert_eq!(data, pa.collect_from_keyed_list());

        // Empty the list
        let _ = pa
            .create_render()
            .keyed_list_with_render(&empty, mode, "span", get_key, render);
        assert_eq!(
            Some(""),
            pa.root.ws_element().as_ref().text_content().as_deref()
        );
        assert_eq!(empty, pa.collect_from_keyed_list());

        // Add back
        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"];
        let _ = pa
            .create_render()
            .keyed_list_with_render(&data, mode, "span", get_key, render);
        assert_eq!(data, pa.collect_from_keyed_list());
        assert_eq!(
            Some("abcdefghijk"),
            pa.root.ws_element().as_ref().text_content().as_deref()
        );

        // Forward
        let data = vec!["a", "i", "b", "c", "d", "e", "f", "g", "h", "j", "k"];
        let _ = pa
            .create_render()
            .keyed_list_with_render(&data, mode, "span", get_key, render);
        assert_eq!(data, pa.collect_from_keyed_list());
        assert_eq!(
            Some("aibcdefghjk"),
            pa.root.ws_element().as_ref().text_content().as_deref()
        );

        // Backward
        let data = vec!["a", "i", "c", "d", "e", "f", "g", "h", "b", "j", "k"];
        let _ = pa
            .create_render()
            .keyed_list_with_render(&data, mode, "span", get_key, render);
        assert_eq!(data, pa.collect_from_keyed_list());
        assert_eq!(
            Some("aicdefghbjk"),
            pa.root.ws_element().as_ref().text_content().as_deref()
        );

        // Swap
        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"];
        let _ = pa
            .create_render()
            .keyed_list_with_render(&data, mode, "span", get_key, render);
        assert_eq!(data, pa.collect_from_keyed_list());
        assert_eq!(
            Some("abcdefghijk"),
            pa.root.ws_element().as_ref().text_content().as_deref()
        );

        // Remove middle
        let data = vec!["a", "b", "c", "d", "i", "j", "k"];
        let _ = pa
            .create_render()
            .keyed_list_with_render(&data, mode, "span", get_key, render);
        assert_eq!(data, pa.collect_from_keyed_list());
        assert_eq!(
            Some("abcdijk"),
            pa.root.ws_element().as_ref().text_content().as_deref()
        );

        // Insert middle
        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"];
        let _ = pa
            .create_render()
            .keyed_list_with_render(&data, mode, "span", get_key, render);
        assert_eq!(data, pa.collect_from_keyed_list());
        assert_eq!(
            Some("abcdefghijk"),
            pa.root.ws_element().as_ref().text_content().as_deref()
        );

        // Remove start
        let data = vec!["d", "e", "f", "g", "h", "i", "j", "k"];
        let _ = pa
            .create_render()
            .keyed_list_with_render(&data, mode, "span", get_key, render);
        assert_eq!(data, pa.collect_from_keyed_list());
        assert_eq!(
            Some("defghijk"),
            pa.root.ws_element().as_ref().text_content().as_deref()
        );

        // Insert start
        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"];
        let _ = pa
            .create_render()
            .keyed_list_with_render(&data, mode, "span", get_key, render);
        assert_eq!(data, pa.collect_from_keyed_list());
        assert_eq!(
            Some("abcdefghijk"),
            pa.root.ws_element().as_ref().text_content().as_deref()
        );

        // Remove end
        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h"];
        let _ = pa
            .create_render()
            .keyed_list_with_render(&data, mode, "span", get_key, render);
        assert_eq!(data, pa.collect_from_keyed_list());
        assert_eq!(
            Some("abcdefgh"),
            pa.root.ws_element().as_ref().text_content().as_deref()
        );

        // Append end
        let data = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"];
        let _ = pa
            .create_render()
            .keyed_list_with_render(&data, mode, "span", get_key, render);
        assert_eq!(data, pa.collect_from_keyed_list());
        assert_eq!(
            Some("abcdefghijk"),
            pa.root.ws_element().as_ref().text_content().as_deref()
        );
    }
}
