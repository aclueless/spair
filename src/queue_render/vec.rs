use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::UnwrapThrowExt;

pub trait ListRender<I> {
    fn render(&mut self, items: &[I], diffs: &[Diff<I>]);
    fn unmounted(&self) -> bool;
}

pub struct QrVec<I>(Rc<RefCell<QrVecContent<I>>>);

impl<I> Clone for QrVec<I> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<I> QrVec<I> {
    pub(crate) fn content(&self) -> &Rc<RefCell<QrVecContent<I>>> {
        &self.0
    }
}

pub struct QrVecContent<I> {
    values: Vec<I>,
    a_render_is_queued: bool,
    diffs: Vec<Diff<I>>,
    // TODO: remove dropped renders
    renders: Vec<Box<dyn ListRender<I>>>,
}

impl<I> QrVecContent<I> {
    pub fn add_render(&mut self, r: Box<dyn ListRender<I>>) {
        self.renders.push(r);
    }

    fn render(&mut self) {
        for r in self.renders.iter_mut() {
            r.render(&self.values, &self.diffs);
        }
        self.diffs.clear();
        self.a_render_is_queued = false;
    }

    fn need_to_queue_a_render(&mut self) -> bool {
        if self.a_render_is_queued {
            return false;
        }
        self.a_render_is_queued = true;
        true
    }
}

// To support multi-changes, we have to store a copy of the item for some change here.
pub enum Diff<I> {
    New,
    Push(I),
    Pop,
    Insert { index: usize, value: I },
    RemoveAtIndex(usize),
    ReplaceAt { index: usize, new_value: I },
    Move { old_index: usize, new_index: usize },
    Swap { index_1: usize, index_2: usize },
    Clear,
}

impl<I: 'static> QrVec<I> {
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

    fn perform<O>(&self, action: impl FnOnce(&mut QrVecContent<I>) -> O) -> O {
        let (queue_me, output) = {
            let mut content = self
                .0
                .try_borrow_mut()
                .expect_throw("queue_render::vec::QrVec::push");
            let output = (action)(&mut content);
            let queue_me = content.need_to_queue_a_render();
            (queue_me, output)
        };
        if queue_me {
            self.queue_a_render();
        }
        output
    }

    pub fn new_values(&self, values: Vec<I>) {
        self.perform(|content| {
            content.diffs.push(Diff::New);
            content.values = values;
        })
    }

    pub fn push(&self, item: I) {
        self.perform(|content| {
            content.diffs.push(Diff::Push(item.clone()));
            content.values.push(item);
        });
    }

    pub fn pop(&self) -> Option<I> {
        self.perform(|content| {
            content.diffs.push(Diff::Pop);
            content.values.pop()
        })
    }

    pub fn insert_at(&self, index: usize, item: I) -> Result<(), QrVecError> {
        self.perform(|content| {
            if index > content.values.len() {
                return Err(QrVecError::IndexOutBounds(index));
            } else if index == content.values.len() {
                content.diffs.push(Diff::Push(item.clone()));
                content.values.push(item);
                Ok(())
            } else {
                content.diffs.push(Diff::Insert {
                    index,
                    value: item.clone(),
                });
                content.values.insert(index, item);
                Ok(())
            }
        })
    }

    pub fn remove_at(&self, index: usize) -> Option<I> {
        self.perform(|content| {
            if index >= content.values.len() {
                return None;
            }
            content.diffs.push(Diff::RemoveAtIndex(index));
            Some(content.values.remove(index))
        })
    }

    pub fn replace_at(&self, index: usize, mut item: I) -> Option<I> {
        self.perform(|content| {
            if index >= content.values.len() {
                return None;
            }
            content.diffs.push(Diff::ReplaceAt {
                index,
                new_value: item.clone(),
            });
            std::mem::swap(&mut item, &mut content.values[index]);
            Some(item)
        })
    }

    pub fn r#move(&self, old_index: usize, new_index: usize) -> Result<(), QrVecError> {
        self.perform(|content| {
            if old_index >= content.values.len() {
                return Err(QrVecError::IndexOutBounds(old_index));
            }
            if new_index >= content.values.len() {
                return Err(QrVecError::IndexOutBounds(new_index));
            }
            content.diffs.push(Diff::Move {
                old_index,
                new_index,
            });
            let item = content.values.remove(old_index);
            content.values.insert(new_index, item);
            Ok(())
        })
    }

    pub fn swap(&self, index_1: usize, index_2: usize) -> Result<(), QrVecError> {
        self.perform(|content| {
            if index_1 >= content.values.len() {
                return Err(QrVecError::IndexOutBounds(index_1));
            }
            if index_2 >= content.values.len() {
                return Err(QrVecError::IndexOutBounds(index_2));
            }
            content.diffs.push(Diff::Swap { index_1, index_2 });
            content.values.swap(index_1, index_2);
            Ok(())
        })
    }

    pub fn clear(&self) {
        self.perform(|content| {
            if content.values.is_empty() {
                return;
            }
            content.diffs.push(Diff::Clear);
            content.values.clear();
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum QrVecError {
    #[error("Index value = {0} is out of bounds")]
    IndexOutBounds(usize),
}
