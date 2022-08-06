use wasm_bindgen::UnwrapThrowExt;

use std::cell::RefCell;
use std::rc::Rc;

compile_error!(
    r#"
QueueRenderingNode may need {
    start_flag,
    end_flat,
} because the incremental dom does not no how a QueueRenderingNode work
"#
);

use crate::dom::queue_render::text::MapTextNode;

pub struct Value<T>(Rc<RefCell<ValueContent<T>>>);
pub struct MapValue<C, T, U, F>
where
    F: Fn(&C, &T) -> U,
{
    value: Value<T>,
    map: F,
    phantom: std::marker::PhantomData<dyn Fn(C, T) -> U>,
}

struct ValueContent<T> {
    value: T,
    // TODO: Removed dropped renders
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
        match self.0.try_borrow() {
            Ok(this) => {
                for r in this.renders.iter() {
                    r.render(&this.value);
                }
            }
            Err(e) => log::error!("{}", e),
        }
    }

    pub fn map<C, U, F>(&self, map: F) -> MapValue<C, T, U, F>
    where
        F: Fn(&C, &T) -> U,
    {
        MapValue {
            value: self.clone(),
            map,
            phantom: std::marker::PhantomData,
        }
    }
}

pub trait QueueRendering<T> {
    fn render(&self, t: &T);
    fn dropped(&self) -> bool;
}

impl<C, T> crate::Render<C> for &Value<T>
where
    C: super::Component,
    T: ToString,
{
    fn render(self, nodes: crate::Nodes<C>) {
        if let Some(text_node) = nodes.create_queue_rendering_text() {
            match self.0.try_borrow_mut() {
                Ok(mut this) => {
                    text_node.update_text(&this.value.to_string());
                    this.renders.push(Box::new(text_node));
                }
                Err(e) => log::error!("{}", e),
            }
        }
    }
}

impl<C, T, U, F> crate::Render<C> for MapValue<C, T, U, F>
where
    C: super::Component,
    T: 'static + ToString,
    U: 'static + ToString,
    F: 'static + Fn(&C, &T) -> U,
{
    fn render(self, nodes: crate::Nodes<C>) {
        let state = nodes.state();
        if let Some(text_node) = nodes.create_queue_rendering_text() {
            let map_node = MapTextNode::new(text_node, self.map);
            match self.value.0.try_borrow_mut() {
                Ok(mut this) => {
                    let u = map_node.map_with_state(state, &this.value);
                    map_node.update_text(&u.to_string());
                    this.renders.push(Box::new(map_node));
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
