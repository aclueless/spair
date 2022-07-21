use wasm_bindgen::UnwrapThrowExt;

use std::cell::RefCell;
use std::rc::Rc;

pub struct Value<T>(Rc<RefCell<ValueContent<T>>>);
struct ValueContent<T> {
    value: T,
    renders: Vec<Box<dyn QueueRendering<T>>>,
}

impl<T> From<T> for Value<T> {
    fn from(t: T) -> Self {
        Value(Rc::new(RefCell::new(ValueContent {
            value: t,
            renders: Vec::new(),
        })))
    }
}

impl<T: 'static + PartialEq + Copy> Value<T> {
    pub fn get(&self) -> T {
        self.0.try_borrow().expect_throw("Borrow for getting T").value
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
        match self.0.try_borrow() {
            Ok(this) => {
                for r in this.renders.iter() {
                    r.render(&this.value);
                }
            }
            Err(e) => log::error!("{}", e),
        }
    }
}

pub trait QueueRendering<T> {
    fn render(&self, t: &T);
}

impl<C, T> crate::Render<C> for &Value<T>
where
    C: super::Component,
    T: ToString,
{
    fn render(self, nodes: crate::Nodes<C>) {
        if let Some(tn) = nodes.create_queue_rendering_text() {
            match self.0.try_borrow_mut() {
                Ok(mut this) => {
                    tn.update_text(&this.value.to_string());
                    this.renders.push(Box::new(tn));
                }
                Err(e) => log::error!("{}", e),
            }
        }
    }
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
