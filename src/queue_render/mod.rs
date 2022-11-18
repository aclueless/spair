use std::{cell::RefCell, collections::VecDeque};

pub mod base;
pub mod dom;
pub mod html;
#[cfg(feature = "svg")]
pub mod svg;
pub mod val;
pub mod vec;

type FnMap<T, U> = Box<dyn Fn(&T) -> U>;
type FnMapC<C, T, U> = Box<dyn Fn(&C, &T) -> U>;

struct RenderQueue {
    queue: RefCell<VecDeque<Box<dyn FnOnce()>>>,
}

thread_local! {
    static RENDER_QUEUE: RenderQueue = RenderQueue {
        queue: RefCell::new(VecDeque::new())
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
