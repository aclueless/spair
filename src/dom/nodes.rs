use wasm_bindgen::UnwrapThrowExt;

#[derive(Default, Clone)]
pub struct NodeList(Vec<Node>);

impl NodeList {
    pub(crate) fn count(&self) -> usize {
        self.0.len()
    }
    fn clear_raw(&mut self) {
        self.0.clear()
    }

    fn clear(&mut self, parent: &web_sys::Node) {
        self.0.drain(..).for_each(|mut node| node.clear(parent));
    }

    pub fn append_to(&self, parent: &web_sys::Node) {
        self.0.iter().for_each(|node| node.append_to(parent));
    }

    fn get_element<'a, C>(
        &'a mut self,
        extra: &super::Extra<'a, C>,
        status: super::ElementStatus,
    ) -> super::ElementHandle<'a, C> {
        match self
            .0
            .get_mut(extra.index)
            .expect_throw("Expect an element node at the given index")
        {
            Node::Element(element) => element.create_handle(extra.comp, status),
            _ => panic!("Why not an element?"),
        }
    }

    fn element<'a, C>(
        &'a mut self,
        tag: &str,
        extra: &super::Extra<'a, C>,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) -> super::ElementHandle<'a, C> {
        let status = if extra.index == self.0.len() {
            self.create_new_element(tag, parent, next_sibling);
            super::ElementStatus::JustCreated
        } else {
            extra.status
        };
        self.get_element(extra, status)
    }

    pub fn clear_after(&mut self, index: usize, parent: &web_sys::Node) {
        if index < self.0.len() {
            if index == 0 {
                parent.set_text_content(None);
                self.clear_raw();
            } else {
                self.0
                    .drain(index..)
                    .for_each(|mut node| node.clear(parent));
            }
        }
    }

    pub fn item_for_list<'a, C>(
        &'a mut self,
        tag: &str,
        extra: &super::Extra<'a, C>,
        parent: &web_sys::Node,
    ) -> super::ElementHandle<'a, C> {
        let item_count = self.0.len();
        let status = if extra.index < item_count {
            super::ElementStatus::Existing
        } else if item_count == 0 {
            // Assumption:
            //  * A list is the only thing in the parent element.
            //  * New item only added to the end of the list.
            // => next_sibling = None
            self.create_new_element(tag, parent, None);
            super::ElementStatus::JustCreated
        } else {
            self.create_new_element_by_cloning_first_item(parent);
            super::ElementStatus::JustCloned
        };
        self.get_element(extra, status)
    }

    fn create_new_element(
        &mut self,
        tag: &str,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) {
        let element = super::Element::new(tag);
        element.insert_before(parent, next_sibling);
        self.0.push(Node::Element(element));
    }

    fn create_new_element_by_cloning_first_item(&mut self, parent: &web_sys::Node) {
        let element = self.0[0].clone();
        // should be append to or insert to?
        element.append_to(parent);
        self.0.push(element);
    }

    fn match_if<'a, C>(
        &'a mut self,
        extra: &super::Extra<'a, C>,
        parent: &'a web_sys::Node,
    ) -> MatchIfHandle<'a, C> {
        if extra.index == self.0.len() {
            let mi = MatchIf::default();
            mi.append_to(parent);
            self.0.push(Node::MatchIf(mi));
        }

        match self
            .0
            .get_mut(extra.index)
            .expect_throw("Expect a match/if arm node at the given index")
        {
            Node::MatchIf(mi) => MatchIfHandle {
                mi,
                parent,
                comp: extra.comp,
            },
            _ => panic!("Why not a match/if arm?"),
        }
    }

    fn static_text(
        &mut self,
        index: usize,
        text: &str,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) {
        if index == self.0.len() {
            self.add_text_node(text, parent, next_sibling);
        }
    }

    fn update_text(
        &mut self,
        index: usize,
        text: &str,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) {
        if index == self.0.len() {
            self.add_text_node(text, parent, next_sibling);
        } else {
            match self
                .0
                .get_mut(index)
                .expect_throw("Expect a text node at the given index")
            {
                Node::Text(text_node) => text_node.update_text(text),
                _ => panic!("Why not a text node?"),
            }
        }
    }

    fn add_text_node(
        &mut self,
        text: &str,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) {
        let text = super::Text::new(text);
        text.insert_before(parent, next_sibling);
        self.0.push(Node::Text(text));
    }
}

#[derive(Clone)]
enum Node {
    Element(super::Element),
    Text(super::Text),
    MatchIf(MatchIf),
}

impl Node {
    fn clear(&mut self, parent: &web_sys::Node) {
        match self {
            Node::Element(element) => element.clear(parent),
            Node::Text(text) => text.clear(parent),
            Node::MatchIf(mi) => mi.clear(parent),
        }
    }

    fn append_to(&self, parent: &web_sys::Node) {
        match self {
            Node::Element(element) => element.append_to(parent),
            Node::Text(text) => text.append_to(parent),
            Node::MatchIf(mi) => mi.append_to(parent),
        }
    }
}

pub struct MatchIf {
    active_index: Option<usize>,
    // `end_node` purpose is to mark the boundary for the arm content, it is useful when
    // users switch between match/if arms and we have to clear the current arm before
    // render new arm.
    end_node: web_sys::Node,
    nodes: NodeList,
}

impl Clone for MatchIf {
    fn clone(&self) -> Self {
        // a match/if arm should not be cloned?
        Default::default()
    }
}

impl Default for MatchIf {
    fn default() -> Self {
        let end_node = crate::utils::document()
            .create_comment("Mark the end of a match/if arm")
            .into();
        Self {
            active_index: None,
            end_node,
            nodes: NodeList::default(),
        }
    }
}

impl MatchIf {
    pub fn clear(&mut self, parent: &web_sys::Node) {
        self.nodes.clear(parent);
    }

    pub fn append_to(&self, parent: &web_sys::Node) {
        self.nodes.append_to(parent);
        parent
            .append_child(&self.end_node)
            .expect_throw("Unable to append a match/if arm's end node to its expected parent");
    }
}

pub struct MatchIfHandle<'a, C> {
    parent: &'a web_sys::Node,
    mi: &'a mut MatchIf,
    comp: &'a crate::component::Comp<C>,
}

impl<'a, C> MatchIfHandle<'a, C> {
    pub fn render_on_arm_index(mut self, index: usize) -> super::Nodes<'a, C> {
        let status = if Some(index) != self.mi.active_index {
            self.mi.nodes.clear(self.parent.as_ref());
            self.mi.active_index = Some(index);
            super::ElementStatus::JustCreated
        } else {
            super::ElementStatus::Existing
        };
        super::Nodes::new(
            &mut self.mi.nodes,
            super::Extra {
                comp: self.comp,
                status,
                index: 0,
            },
            self.parent,
            Some(&self.mi.end_node),
        )
    }
}

pub(crate) struct NodeListHandle<'a, C> {
    parent: &'a web_sys::Node,
    next_sibling: Option<&'a web_sys::Node>,
    nodes: &'a mut super::NodeList,
    extra: super::Extra<'a, C>,
}

impl<'a, C> NodeListHandle<'a, C> {
    pub fn from_handle(mut handle: super::ElementHandle<'a, C>) -> Self {
        handle.extra.index = 0;
        Self {
            parent: handle.element.ws_element.as_ref(),
            next_sibling: None,
            nodes: &mut handle.element.nodes,
            extra: handle.extra,
        }
    }
}

pub struct StaticNodes<'a, C>(NodeListHandle<'a, C>);

impl<'a, C> StaticNodes<'a, C> {
    pub(super) fn from_handle(handle: super::ElementHandle<'a, C>) -> Self {
        Self(NodeListHandle::from_handle(handle))
    }

    pub fn comp(&self) -> crate::component::Comp<C> {
        self.0.extra.comp.clone()
    }

    pub fn nodes(self) -> Nodes<'a, C> {
        Nodes(self.0)
    }

    pub fn render(self, value: impl crate::renderable::Render<C>) -> Self {
        value.render(self.nodes()).static_nodes()
    }

    pub fn r#static(self, value: impl crate::renderable::StaticRender<C>) -> Self {
        value.render(self)
    }

    pub(crate) fn static_text(mut self, text: &str) -> Self {
        self.0
            .nodes
            .static_text(self.0.extra.index, text, self.0.parent, self.0.next_sibling);
        self.0.extra.index += 1;
        self
    }
}

pub struct Nodes<'a, C>(NodeListHandle<'a, C>);

impl<'a, C> Nodes<'a, C> {
    pub(super) fn new(
        nodes: &'a mut super::NodeList,
        extra: super::Extra<'a, C>,
        parent: &'a web_sys::Node,
        next_sibling: Option<&'a web_sys::Node>,
    ) -> Self {
        Self(NodeListHandle {
            nodes,
            extra,
            parent,
            next_sibling,
        })
    }
    pub(super) fn from_handle(handle: super::ElementHandle<'a, C>) -> Self {
        Self(NodeListHandle::from_handle(handle))
    }

    pub fn comp(&self) -> crate::component::Comp<C> {
        self.0.extra.comp.clone()
    }

    pub fn static_nodes(self) -> StaticNodes<'a, C> {
        StaticNodes(self.0)
    }

    pub fn render(self, value: impl crate::renderable::Render<C>) -> Self {
        value.render(self)
    }

    pub fn r#static(self, value: impl crate::renderable::StaticRender<C>) -> Self {
        value.render(self.static_nodes()).nodes()
    }

    pub(crate) fn static_text(mut self, text: &str) -> Self {
        self.0
            .nodes
            .static_text(self.0.extra.index, text, self.0.parent, self.0.next_sibling);
        self.0.extra.index += 1;
        self
    }

    pub(crate) fn update_text(mut self, text: &str) -> Self {
        self.0
            .nodes
            .update_text(self.0.extra.index, text, self.0.parent, self.0.next_sibling);
        self.0.extra.index += 1;
        self
    }
}

macro_rules! create_methods_for_tags {
    ($($tag:ident)+) => {
        $(
            fn $tag(self, f: impl FnOnce(super::ElementHandle<C>)) -> Self {
                self.render_element(stringify!($tag), f)
            }
        )+
    }
}

pub trait DomBuilder<C>: Sized {
    /// Use this method the compiler complains about expected `()` but found something else and you don't want to add `;`
    fn done(self) {}
    fn require_render(&self) -> bool;
    fn next_index(&mut self) {}
    fn get_element_and_increase_index(&mut self, tag: &str) -> super::ElementHandle<C>;
    fn get_match_if_and_increase_index(&mut self) -> MatchIfHandle<C>;

    fn match_if(mut self, f: impl FnOnce(MatchIfHandle<C>)) -> Self {
        f(self.get_match_if_and_increase_index());
        self
    }

    fn render_element(mut self, tag: &str, f: impl FnOnce(super::ElementHandle<C>)) -> Self {
        if self.require_render() {
            f(self.get_element_and_increase_index(tag));
        } else {
            self.next_index();
        }
        self
    }

    create_methods_for_tags! {
        a
        button
        div
        footer
        header h1 h2 h3 h4 h5 h6
        input
        label
        li
        p
        section
        span
        strong
        ul
    }
}

impl<'a, C> DomBuilder<C> for StaticNodes<'a, C> {
    fn require_render(&self) -> bool {
        self.0.extra.status == super::ElementStatus::JustCreated
    }

    fn next_index(&mut self) {
        self.0.extra.index += 1;
    }

    fn get_element_and_increase_index(&mut self, tag: &str) -> super::ElementHandle<C> {
        let e = self
            .0
            .nodes
            .element(tag, &self.0.extra, self.0.parent, self.0.next_sibling);
        self.0.extra.index += 1;
        e
    }

    fn get_match_if_and_increase_index(&mut self) -> MatchIfHandle<C> {
        let mi = self.0.nodes.match_if(&self.0.extra, self.0.parent);
        self.0.extra.index += 1;
        mi
    }
}

impl<'a, C> DomBuilder<C> for Nodes<'a, C> {
    fn require_render(&self) -> bool {
        true
    }

    fn get_element_and_increase_index(&mut self, tag: &str) -> super::ElementHandle<C> {
        let e = self
            .0
            .nodes
            .element(tag, &self.0.extra, self.0.parent, self.0.next_sibling);
        self.0.extra.index += 1;
        e
    }

    fn get_match_if_and_increase_index(&mut self) -> MatchIfHandle<C> {
        let mi = self.0.nodes.match_if(&self.0.extra, self.0.parent);
        self.0.extra.index += 1;
        mi
    }
}
