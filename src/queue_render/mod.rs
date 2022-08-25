pub mod base;
pub mod dom;
pub mod html;

use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::UnwrapThrowExt;

pub struct Value<T>(Rc<RefCell<ValueContent<T>>>);
pub struct MapValue<C, T, U> {
    value: Value<T>,
    fn_map: Box<dyn Fn(&C, &T) -> U>,
}

impl<C, T, U> MapValue<C, T, U> {
    pub fn into_parts(self) -> (Value<T>, Box<dyn Fn(&C, &T) -> U>) {
        (self.value, self.fn_map)
    }
}

pub struct ValueContent<T> {
    value: T,
    // TODO: Removed dropped renders
    renders: Vec<Box<dyn QueueRender<T>>>,
}

impl<T> ValueContent<T> {
    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn add_render(&mut self, r: Box<dyn QueueRender<T>>) {
        self.renders.push(r);
    }

    fn render(&mut self) {
        for r in self.renders.iter_mut() {
            r.render(&self.value)
        }
    }
}

impl<T> From<T> for Value<T> {
    fn from(t: T) -> Self {
        Value(Rc::new(RefCell::new(ValueContent {
            value: t,
            renders: Vec::new(),
        })))
    }
}

impl<T: 'static> Value<T> {
    pub(crate) fn content(&self) -> &Rc<RefCell<ValueContent<T>>> {
        &self.0
    }
}

impl<T: 'static + PartialEq + Copy> Value<T> {
    pub fn get(&self) -> T {
        self.0
            .try_borrow()
            .expect_throw("Borrow for getting T")
            .value
    }
}

impl<T: 'static + PartialEq> Value<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }

    fn queue_me(&self, queue_me: bool) {
        if queue_me {
            let this = self.clone();
            queue_render(move || this.render());
        }
    }

    pub fn set(&mut self, t: T) {
        let queue_me = match self.0.try_borrow_mut() {
            Ok(mut this) if t != this.value => {
                this.value = t;
                true
            }
            Ok(_) => false,
            Err(e) => {
                log::error!("{}", e);
                return;
            }
        };
        self.queue_me(queue_me);
    }

    pub fn set_with(&mut self, ft: impl FnOnce(&T) -> T) {
        let queue_me = match self.0.try_borrow_mut() {
            Ok(mut this) => {
                let t = ft(&this.value);
                if t == this.value {
                    false
                } else {
                    this.value = t;
                    true
                }
            }
            Err(e) => {
                log::error!("{}", e);
                return;
            }
        };
        self.queue_me(queue_me);
    }

    fn render(&self) {
        match self.0.try_borrow_mut() {
            Ok(mut this) => this.render(),
            Err(e) => log::error!("queue render error: {}", e),
        }
    }

    pub fn map<C, U, F>(&self, fn_map: F) -> MapValue<C, T, U>
    where
        F: 'static + Fn(&C, &T) -> U,
    {
        MapValue {
            value: self.clone(),
            fn_map: Box::new(fn_map),
        }
    }
}

pub trait QueueRender<T> {
    fn render(&mut self, t: &T);
    fn unmounted(&self) -> bool;
}

struct RenderQueue {
    queue: RefCell<std::collections::VecDeque<Box<dyn FnOnce()>>>,
}

thread_local! {
    static RENDER_QUEUE: RenderQueue = RenderQueue {
        queue: RefCell::new(std::collections::VecDeque::new())
    };
}

fn queue_render(fn_render: impl FnOnce() + 'static) {
    RENDER_QUEUE.with(|rq| rq.add(Box::new(fn_render)));
}

impl RenderQueue {
    fn add(&self, f: Box<dyn FnOnce()>) {
        self.queue.borrow_mut().push_back(f);
    }

    fn take(&self) -> Option<Box<dyn FnOnce()>> {
        self.queue.borrow_mut().pop_front()
    }

    fn execute(&self) {
        while let Some(f) = self.take() {
            f();
        }
    }
}

pub fn execute_render_queue() {
    RENDER_QUEUE.with(|uq| uq.execute());
}
