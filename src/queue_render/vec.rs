use std::{
    cell::{Ref, RefCell, RefMut},
    cmp::Ordering,
    ops::{Deref, DerefMut},
    rc::Rc,
};
use wasm_bindgen::UnwrapThrowExt;

use super::val::{QrVal, QueueRender};

pub trait ListRender<I: Clone> {
    fn render(&mut self, items: &[I], diffs: Vec<Diff<I>>);
    #[allow(dead_code)]
    fn unmounted(&self) -> bool;
}

/// Currently, QrVec rendering is not flexible. For example,
/// it is impossible for you to make use of QrVec as a list of items
/// and render the items into multi-rows of <div>s.
pub struct QrVec<I: Clone>(Rc<RefCell<QrVecContent<I>>>);

impl<I: Clone> Clone for QrVec<I> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<I: Clone> QrVec<I> {
    pub(crate) fn content(&self) -> &Rc<RefCell<QrVecContent<I>>> {
        &self.0
    }
}

pub struct QrVecContent<I: Clone> {
    values: Vec<I>,
    a_render_is_queued: bool,
    diffs: Vec<Diff<I>>,
    // TODO: remove dropped renders
    renders: Vec<Box<dyn ListRender<I>>>,
}

impl<I: Clone> QrVecContent<I> {
    pub(crate) fn add_render(&mut self, r: Box<dyn ListRender<I>>) {
        self.renders.push(r);
    }

    fn render(&mut self) {
        if self.renders.is_empty() {
            return;
        }
        let before_last = self.renders.len() - 1;
        for r in self.renders[..before_last].iter_mut() {
            r.render(&self.values, self.diffs.clone());
        }
        if let Some(r) = self.renders.last_mut() {
            let mut diffs = Vec::new();
            std::mem::swap(&mut diffs, &mut self.diffs);
            r.render(&self.values, diffs);
        }
        self.a_render_is_queued = false;
    }

    fn need_to_queue_a_render(&mut self) -> bool {
        if self.a_render_is_queued {
            return false;
        }
        self.a_render_is_queued = true;
        true
    }

    pub fn iter(&self) -> impl Iterator<Item = &I> {
        self.values.iter()
    }

    pub fn reserve(&mut self, additional: usize) {
        self.values.reserve(additional);
    }

    pub fn new_values(&mut self, values: Vec<I>) {
        self.diffs.push(Diff::New);
        self.values = values;
    }

    pub fn push(&mut self, item: I) {
        self.diffs.push(Diff::Push {
            value: item.clone(),
        });
        self.values.push(item);
    }

    pub fn pop(&mut self) -> Option<I> {
        self.diffs.push(Diff::Pop);
        self.values.pop()
    }

    pub fn insert_at(&mut self, index: usize, item: I) -> Result<(), QrVecError> {
        match index.cmp(&self.values.len()) {
            Ordering::Greater => Err(QrVecError::IndexOutBounds(index)),
            Ordering::Equal => {
                self.diffs.push(Diff::Push {
                    value: item.clone(),
                });
                self.values.push(item);
                Ok(())
            }
            Ordering::Less => {
                self.diffs.push(Diff::Insert {
                    index,
                    value: item.clone(),
                });
                self.values.insert(index, item);
                Ok(())
            }
        }
    }

    pub fn remove_at(&mut self, index: usize) -> Option<I> {
        if index >= self.values.len() {
            return None;
        }
        self.diffs.push(Diff::RemoveAt { index });
        Some(self.values.remove(index))
    }

    pub fn replace_at(&mut self, index: usize, mut item: I) -> Option<I> {
        if index >= self.values.len() {
            return None;
        }
        self.diffs.push(Diff::ReplaceAt {
            index,
            new_value: item.clone(),
        });
        std::mem::swap(&mut item, &mut self.values[index]);
        Some(item)
    }

    pub fn r#move(&mut self, old_index: usize, new_index: usize) -> Result<(), QrVecError> {
        if old_index >= self.values.len() {
            return Err(QrVecError::IndexOutBounds(old_index));
        }
        if new_index >= self.values.len() {
            return Err(QrVecError::IndexOutBounds(new_index));
        }
        self.diffs.push(Diff::Move {
            old_index,
            new_index,
        });
        let item = self.values.remove(old_index);
        self.values.insert(new_index, item);
        Ok(())
    }

    pub fn swap(&mut self, index_1: usize, index_2: usize) -> Result<(), QrVecError> {
        if index_1 >= self.values.len() {
            return Err(QrVecError::IndexOutBounds(index_1));
        }
        if index_2 >= self.values.len() {
            return Err(QrVecError::IndexOutBounds(index_2));
        }
        self.diffs.push(Diff::Swap { index_1, index_2 });
        self.values.swap(index_1, index_2);
        Ok(())
    }

    pub fn clear(&mut self) {
        if self.values.is_empty() {
            return;
        }
        self.diffs.push(Diff::New);
        self.values.clear();
    }

    fn _request_render_at(&mut self, index: usize) {
        let item = match self.values.get(index) {
            None => return,
            Some(item) => Clone::clone(item),
        };
        self.diffs.push(Diff::Render { index, value: item });
    }

    pub fn request_render_at(&mut self, index: impl RequestRenderIndex<I>) {
        index.perform(self);
    }

    // Copied from https://github.com/Pauan/rust-signals
    pub fn retain(&mut self, filter: impl Fn(&I) -> bool) {
        if self.values.is_empty() {
            return;
        }
        let mut len = self.values.len();
        let mut index = 0;
        let mut removed_indices = Vec::with_capacity(8);
        self.values.retain(|v| {
            let to_retain = filter(v);
            if !to_retain {
                removed_indices.push(index);
            }
            index += 1;
            to_retain
        });
        if self.values.is_empty() {
            self.diffs.push(Diff::New);
            return;
        }
        for removed_index in removed_indices.into_iter().rev() {
            len -= 1;
            if removed_index == len {
                self.diffs.push(Diff::Pop);
            } else {
                self.diffs.push(Diff::RemoveAt {
                    index: removed_index,
                });
            }
        }
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = QrVecItemMut<I>> {
        // I tried many ways to provide an `.iter_mut()` that can record
        // the changes. I always failed because of lifetime issues. So,
        // the last resort is using unsafe.

        // IMI stands for iter-mut-iterator - the iterator returns by this
        // method.

        // `self.diffs` is belong to `self`. `self` will be alive at least
        // until the IMI finish. `self` will never be moved during the
        // executing of the IMI. Therefore, the raw pointer of `self.diffs`
        // is always valid during the lifetime of IMI.

        // `self.diffs` is a Vec. The IMI may push new values to the
        // `self.diffs` (via the raw pointer `QrVecItemMut::diffs`). The
        // `Vec::buf` may have to be reallocated. `Vec::buf` may change,
        // but the `Vec` itself is never moved around during IMI.

        // The final words are "It is safe to use raw pointer to `&mut
        // self.diff` during the lifetime of IMI."
        let diffs = &mut self.diffs;
        self.values
            .iter_mut()
            .enumerate()
            .map(|(index, value)| QrVecItemMut {
                index,
                value,
                diffs,
            })
    }
}

pub struct QrVecItemMut<'a, I: Clone> {
    index: usize,
    value: &'a mut I,
    diffs: *mut Vec<Diff<I>>,
}

impl<'a, I: Clone> QrVecItemMut<'a, I> {
    pub fn modify(&mut self, f: impl FnOnce(&mut I)) {
        f(self.value);
        let diff = Diff::ReplaceAt {
            index: self.index,
            new_value: self.value.clone(),
        };
        // `QrVecContent::iter_mut()` is responsible to make this safe
        // See details in comments in `QrVecContent::iter_mut()`.
        unsafe {
            (*self.diffs).push(diff);
        }
    }
}

pub trait RequestRenderIndex<I: Clone> {
    fn perform(self, qrvec: &mut QrVecContent<I>);
}

impl<I: 'static + Clone> RequestRenderIndex<I> for usize {
    fn perform(self, qrvec: &mut QrVecContent<I>) {
        qrvec._request_render_at(self);
    }
}

impl<I: 'static + Clone> RequestRenderIndex<I> for Option<usize> {
    fn perform(self, qrvec: &mut QrVecContent<I>) {
        if let Some(index) = self {
            qrvec._request_render_at(index);
        }
    }
}

// To support multi-changes, we have to store a copy of the items for some changes here.
#[derive(Debug, Clone)]
pub enum Diff<I: Clone> {
    New,
    Push { value: I },
    Pop,
    Insert { index: usize, value: I },
    RemoveAt { index: usize },
    ReplaceAt { index: usize, new_value: I },
    Move { old_index: usize, new_index: usize },
    Swap { index_1: usize, index_2: usize },
    Render { index: usize, value: I },
}

impl<I: 'static + Clone> QrVec<I> {
    pub(crate) fn check_and_queue_a_render(&self) {
        {
            let mut this = self
                .0
                .try_borrow_mut()
                .expect_throw("queue_render::vec::QrVec::queue_me try_borrow_mut");
            if this.a_render_is_queued {
                return;
            }
            this.a_render_is_queued = true;
        }
        self.queue_a_render();
    }

    pub(crate) fn queue_a_render(&self) {
        let clone_self = self.clone();
        super::queue_render(move || clone_self.render());
    }

    fn render(&self) {
        match self.0.try_borrow_mut() {
            Ok(mut this) => this.render(),
            Err(e) => log::error!("queue_render::vec::QrVec::render {}", e),
        }
    }
}

pub struct QrVecMut<'a, I: 'static + Clone> {
    content: RefMut<'a, QrVecContent<I>>,
    qr_vec: Option<QrVec<I>>,
}

impl<'a, I: 'static + Clone> Deref for QrVecMut<'a, I> {
    type Target = QrVecContent<I>;
    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl<'a, I: 'static + Clone> DerefMut for QrVecMut<'a, I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.content
    }
}

impl<'a, I: 'static + Clone> Drop for QrVecMut<'a, I> {
    fn drop(&mut self) {
        if self.content.need_to_queue_a_render() {
            if let Some(qr_vec) = self.qr_vec.take() {
                super::queue_render(move || qr_vec.render());
            }
        }
    }
}

impl<I: 'static + Clone> Default for QrVec<I> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I: 'static + Clone> QrVec<I> {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(QrVecContent {
            values: Vec::new(),
            a_render_is_queued: false,
            diffs: Vec::new(),
            renders: Vec::new(),
        })))
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(Rc::new(RefCell::new(QrVecContent {
            values: Vec::with_capacity(capacity),
            a_render_is_queued: false,
            diffs: Vec::new(),
            renders: Vec::new(),
        })))
    }

    pub fn with_values(values: Vec<I>) -> Self {
        // Do not have to queue a render because the object is not accessible yet.
        // The first render will be schedule by incremental dom.
        Self(Rc::new(RefCell::new(QrVecContent {
            values,
            a_render_is_queued: false,
            diffs: vec![Diff::New],
            renders: Vec::new(),
        })))
    }

    pub fn get_ref(&self) -> Ref<QrVecContent<I>> {
        self.0
            .try_borrow()
            .expect_throw("queue_render::vec::QrVec::get_ref")
    }

    pub fn get_mut(&self) -> QrVecMut<I> {
        let content = self
            .0
            .try_borrow_mut()
            .expect_throw("queue_render::vec::QrVec::get_mut");
        let qr_vec = self.clone();
        QrVecMut {
            content,
            qr_vec: Some(qr_vec),
        }
    }

    pub fn create_optional_selected_id<T: 'static + Clone>(
        &self,
        is_me: impl Fn(&I, &T) -> bool + 'static,
    ) -> QrVal<Option<T>> {
        let selected_id: QrVal<Option<T>> = None.into();
        match selected_id.content().try_borrow_mut() {
            Ok(mut this) => {
                let osi = OptionalSelectedId {
                    last_selected_id: None,
                    qr_vec: self.clone(),
                    is_me: Box::new(is_me),
                };
                this.add_render(Box::new(osi));
            }
            Err(e) => log::error!("{}", e),
        }
        selected_id
    }
}

type FnIsMe<I, T> = Box<dyn Fn(&I, &T) -> bool>;

struct OptionalSelectedId<I: Clone, T> {
    last_selected_id: Option<T>,
    qr_vec: QrVec<I>,
    is_me: FnIsMe<I, T>,
}

fn get_index<I: Clone, T>(
    selected_id: Option<&T>,
    items: &QrVecMut<I>,
    is_me: &FnIsMe<I, T>,
) -> Option<usize> {
    selected_id
        .as_ref()
        .and_then(|id| items.iter().position(|i| (is_me)(i, id)))
}

impl<I: 'static + Clone, T: Clone> QueueRender<Option<T>> for OptionalSelectedId<I, T> {
    fn render(&mut self, new_selected_id: &Option<T>) {
        let mut items = self.qr_vec.get_mut();
        let old_index = get_index(self.last_selected_id.as_ref(), &items, &self.is_me);
        let new_index = get_index(new_selected_id.as_ref(), &items, &self.is_me);
        self.last_selected_id = new_selected_id.clone();
        items.request_render_at(old_index);
        items.request_render_at(new_index);
    }

    fn unmounted(&self) -> bool {
        // TODO: How to track if the QrVec still need to render?
        false
    }
}

#[derive(thiserror::Error, Debug)]
pub enum QrVecError {
    #[error("Index value = {0} is out of bounds")]
    IndexOutBounds(usize),
}
