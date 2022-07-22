use crate::Component;
use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::UnwrapThrowExt;

pub trait QueueRenderingText {
    fn remove_from(&self, parent: &web_sys::Node);
    fn append_to(&self, parent: &web_sys::Node);
    fn insert_before(&self, parent: &web_sys::Node, next_sibling: Option<&web_sys::Node>);
    //fn get_first_element(&self) -> Option<&super::Element>;
    fn clone_ws_node(&self) -> web_sys::Node;
}

pub enum QueueRendering {
    ActiveTextNode(Box<dyn QueueRenderingText>),
    ClonedWsNode(Option<web_sys::Node>),
}

impl QueueRendering {
    pub fn remove_from(&self, parent: &web_sys::Node) {
        if let Self::ActiveTextNode(n) = self {
            n.remove_from(parent);
        }
    }

    pub fn append_to(&self, parent: &web_sys::Node) {
        if let Self::ActiveTextNode(n) = self {
            n.append_to(parent);
        }
    }

    pub fn insert_before(&self, parent: &web_sys::Node, next_sibling: Option<&web_sys::Node>) {
        if let Self::ActiveTextNode(n) = self {
            n.insert_before(parent, next_sibling);
        }
    }

    pub fn get_first_element(&self) -> Option<&super::Element> {
        match self {
            Self::ActiveTextNode(_) => None,
            Self::ClonedWsNode(_) => None,
        }
    }

    pub fn get_last_element(&self) -> Option<&super::Element> {
        match self {
            Self::ActiveTextNode(_) => None,
            Self::ClonedWsNode(_) => None,
        }
    }
}
impl Clone for QueueRendering {
    fn clone(&self) -> Self {
        match self {
            Self::ActiveTextNode(text) => Self::ClonedWsNode(Some(text.clone_ws_node())),
            Self::ClonedWsNode(wsn) => Self::ClonedWsNode(wsn.as_ref().map(|wsn| {
                wsn.clone_node_with_deep(false).expect_throw(
                    "A cloned web_sys::Node for queue rendering text node in QueueRendering::clone",
                )
            })),
        }
    }
}

pub struct TextNode<C: Component>(Rc<TextNodeInner<C>>);
impl<C: Component> Clone for TextNode<C> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub struct TextNodeInner<C: Component> {
    comp: crate::Comp<C>,
    dropped: Cell<bool>,
    ws_node: web_sys::Node,
}

impl<C: Component> TextNode<C> {
    fn new(comp: crate::Comp<C>) -> Self {
        Self(Rc::new(TextNodeInner {
            comp,
            dropped: Cell::new(false),
            ws_node: crate::utils::document().create_text_node("").into(),
        }))
    }

    fn with_cloned_node(ws_node: web_sys::Node, comp: crate::Comp<C>) -> Self {
        Self(Rc::new(TextNodeInner {
            comp,
            dropped: Cell::new(false),
            ws_node,
        }))
    }

    pub fn update_text(&self, text: &str) {
        self.0.ws_node.set_text_content(Some(text)); //.expect_throw();
    }
}

impl<C: Component> QueueRenderingText for TextNode<C> {
    fn remove_from(&self, parent: &web_sys::Node) {
        self.0.dropped.set(true);
        parent
            .remove_child(&self.0.ws_node)
            .expect_throw("Unable to remove a child Element from its parent");
    }

    fn append_to(&self, parent: &web_sys::Node) {
        parent
            .append_child(&self.0.ws_node)
            .expect_throw("Unable to append a child Text to its expected parent");
    }

    fn insert_before(&self, parent: &web_sys::Node, next_sibling: Option<&web_sys::Node>) {
        parent
            .insert_before(&self.0.ws_node, next_sibling)
            .expect_throw("Unable to insert a child Text to its expected parent");
    }

    // fn get_first_element(&self) -> Option<&super::Element> {
    //     None
    // }

    fn clone_ws_node(&self) -> web_sys::Node {
        self.0.ws_node.clone_node_with_deep(false).expect_throw("A cloned web_sys::Node for queue render text node in QueueRenderingText::clone_ws_node")
    }
}

impl<C: Component, T: ToString> crate::component::queue_render::QueueRendering<T> for TextNode<C> {
    fn render(&self, t: &T) {
        self.update_text(&t.to_string());
    }
}

impl<'a, C: crate::component::Component> super::nodes::NodeListUpdater<'a, C> {
    fn create_queue_rendering_text(&mut self) -> Option<TextNode<C>> {
        let tn = if self.index == self.nodes.count() {
            let tn = super::queue_render::TextNode::new(self.comp());
            self.parent
                .insert_before(&tn.0.ws_node, self.next_sibling)
                .expect_throw("Unable to insert queue rendering text node to parent");
            self.nodes.0.push(super::nodes::Node::QueueRendering(
                QueueRendering::ActiveTextNode(Box::new(tn.clone())),
            ));
            Some(tn)
        } else {
            let comp = self.comp();
            let rs = match self.nodes.0.get_mut(self.index) {
                Some(super::nodes::Node::QueueRendering(qr)) => match qr {
                    QueueRendering::ActiveTextNode(_) => None,
                    QueueRendering::ClonedWsNode(wsn) => match wsn.take() {
                        Some(wsn) => {
                            let tn = TextNode::with_cloned_node(wsn, comp);
                            *qr = QueueRendering::ActiveTextNode(Box::new(tn.clone()));
                            Some(tn)
                        }
                        None => None,
                    },
                },
                _ => panic!("Why not a queue rendering?"),
            };
            rs
        };
        self.index += 1;
        tn
    }
}

impl<'n, 'h, C: crate::component::Component> super::Nodes<'n, 'h, C> {
    pub fn create_queue_rendering_text(self) -> Option<TextNode<C>> {
        self.0.u.create_queue_rendering_text()
    }
}

pub struct MapTextNode<C, T, U, F>
where
    C: Component,
    F: Fn(&C, &T) -> U,
{
    text_node: TextNode<C>,
    map: F,
    phantom: std::marker::PhantomData<dyn Fn(C, T) -> U>,
}

impl<C, T, U, F> MapTextNode<C, T, U, F>
where
    C: Component,
    T: ToString,
    F: Fn(&C, &T) -> U,
{
    pub fn new(text_node: TextNode<C>, map: F) -> Self {
        Self {
            text_node,
            map,
            phantom: std::marker::PhantomData,
        }
    }

    fn map(&self, value: &T) -> U {
        let rc_comp = self.text_node.0.comp.upgrade();
        let comp = rc_comp
            .try_borrow_mut()
            .expect_throw("MapTextNode::map::rc_comp.try_borrow_mut().");
        let state = comp.state();
        (self.map)(state, value)
    }

    pub fn map_with_state(&self, state: &C, value: &T) -> U {
        (self.map)(state, value)
    }

    pub fn update_text(&self, text: &str) {
        self.text_node.update_text(text);
    }
}

impl<C, T, U, F> crate::component::queue_render::QueueRendering<T> for MapTextNode<C, T, U, F>
where
    C: Component,
    T: 'static + ToString,
    U: 'static + ToString,
    F: 'static + Fn(&C, &T) -> U,
{
    fn render(&self, t: &T) {
        let u = self.map(t);
        self.update_text(&u.to_string());
    }
}
