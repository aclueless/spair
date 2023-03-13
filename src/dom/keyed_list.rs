use super::{Element, GroupedNodes};
use std::collections::HashMap;
use uuid::Uuid;
use wasm_bindgen::UnwrapThrowExt;

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum ListEntryKey {
    String(String),
    ISize(isize),
    USize(usize),
    I64(i64),
    U64(u64),
    I32(i32),
    U32(u32),
    Uuid(Uuid),
}

impl From<&String> for ListEntryKey {
    fn from(value: &String) -> Self {
        ListEntryKey::String(value.to_string())
    }
}

impl From<&&str> for ListEntryKey {
    fn from(value: &&str) -> Self {
        ListEntryKey::String(value.to_string())
    }
}

impl PartialEq<ListEntryKey> for String {
    fn eq(&self, other: &ListEntryKey) -> bool {
        match other {
            ListEntryKey::String(value) => value == self,
            _ => false,
        }
    }
}

impl PartialEq<ListEntryKey> for &str {
    fn eq(&self, other: &ListEntryKey) -> bool {
        match other {
            ListEntryKey::String(value) => value == self,
            _ => false,
        }
    }
}

macro_rules! impl_from_and_partial_eq_for_key_type {
    ($($key_type:ident $KeyVariant:ident)+) => {
        $(
            impl From<&$key_type> for ListEntryKey {
                fn from(value: &$key_type) -> Self {
                    ListEntryKey::$KeyVariant(*value)
                }
            }
            impl PartialEq<ListEntryKey> for $key_type {
                fn eq(&self, other: &ListEntryKey) -> bool {
                    match other {
                        ListEntryKey::$KeyVariant(value) => value == self,
                        _ => false,
                    }
                }
            }
        )+
    };
}

impl_from_and_partial_eq_for_key_type! {
    isize ISize
    usize USize
    i64 I64
    u64 U64
    i32 I32
    u32 U32
    Uuid Uuid
}

#[derive(Debug)]
pub struct OldEntry {
    pub index: usize,
    pub group: GroupedNodes,
}

pub struct KeyedEntry {
    pub key: ListEntryKey,
    pub group: GroupedNodes,
}

impl KeyedEntry {
    pub fn new(key: ListEntryKey, group: GroupedNodes) -> Self {
        Self { key, group }
    }
}

pub struct ListEntryTemplate {
    pub rendered: bool,
    pub group: GroupedNodes,
}

// Keyed list occupied the whole parent element. In the example below, there
// is nothing inside the `div` rather than the content rendered by `keyed_list`
// nodes.div(|d| {
//    d.keyed_list(state.items.iter(), |item, d| {});
// })
//
#[derive(Default)]
pub struct KeyedList {
    active: Vec<Option<KeyedEntry>>,
    // The primary reason for the double buffer here is for easy implementation.
    buffer: Vec<Option<KeyedEntry>>,
    template: Option<ListEntryTemplate>,
    old_elements_map: HashMap<ListEntryKey, OldEntry>,
}

impl Clone for KeyedList {
    fn clone(&self) -> Self {
        // No clone for keyed list
        // If cloning is applied to keyed-list, make sure that if the parent status
        // is ElementStatus::Cloned, then every element in the list should also
        // have status=ElementStatus::Cloned
        Self {
            active: Vec::new(),
            buffer: Vec::new(),
            old_elements_map: HashMap::new(),
            template: None,
        }
    }
}

impl KeyedList {
    #[cfg(test)]
    pub fn active_nodes(&self) -> &Vec<Option<KeyedEntry>> {
        &self.active
    }

    pub fn get_first_element(&self) -> Option<&Element> {
        self.active.first().and_then(|i| {
            i.as_ref()
                .and_then(|ke| ke.group.nodes().get_first_element())
        })
    }

    pub fn get_last_element(&self) -> Option<&Element> {
        self.active.last().and_then(|i| {
            i.as_ref()
                .and_then(|ke| ke.group.nodes().get_first_element())
        })
    }

    pub fn require_init_template(&mut self) -> bool {
        match self.template.as_ref() {
            None => {
                self.template = Some(ListEntryTemplate {
                    rendered: false,
                    group: GroupedNodes::new("start of a list entry"),
                });
                true
            }
            Some(t) => !t.rendered,
        }
    }

    #[allow(clippy::type_complexity)]
    pub fn entries_mut(
        &mut self,
    ) -> (
        Option<&mut ListEntryTemplate>,
        &mut Vec<Option<KeyedEntry>>,
        &mut Vec<Option<KeyedEntry>>,
        &mut HashMap<ListEntryKey, OldEntry>,
    ) {
        (
            self.template.as_mut(),
            &mut self.buffer,
            &mut self.active,
            &mut self.old_elements_map,
        )
    }

    // better name?
    pub fn pre_update(&mut self, count: usize) {
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

    pub fn remove_from_dom(self, parent: &web_sys::Node) {
        self.active.into_iter().for_each(|item| {
            item.expect_throw("dom::keyed_list::KeyedList::clear")
                .group
                .remove_from_dom(parent)
        });
    }

    pub fn append_to(&self, parent: &web_sys::Node) {
        self.active.iter().for_each(|item| {
            item.as_ref()
                .expect_throw("dom::keyed_list::KeyedList::append_to")
                .group
                .append_to_parent_with_flag_as_start(parent)
        });
    }

    pub fn insert_before_a_sibling(
        &self,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) {
        self.active.iter().for_each(|item| {
            item.as_ref()
                .expect_throw("dom::keyed_list::KeyedList::insert_before_a_sibling")
                .group
                .insert_before_a_sibling(parent, next_sibling)
        })
    }
}
