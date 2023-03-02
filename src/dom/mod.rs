use wasm_bindgen::UnwrapThrowExt;

mod attributes;
mod element;
mod node;
mod nodes;
mod text;

pub use attributes::*;
pub use element::*;
pub use node::*;
pub use nodes::*;
pub use text::*;

#[cfg(feature = "keyed-list")]
mod keyed_list;
#[cfg(feature = "keyed-list")]
pub use keyed_list::*;

pub trait ElementTag: Copy {
    const NAMESPACE: &'static str;
    fn tag_name(&self) -> &str;
}

pub trait ElementTagExt<'a, C: crate::Component>: ElementTag {
    type Updater;
    fn make_updater(e: crate::render::base::ElementUpdater<'a, C>) -> Self::Updater;
}

pub enum TagName {
    Html(crate::render::html::HtmlTag),
    #[cfg(feature = "svg")]
    Svg(crate::render::svg::SvgTag),
}

pub trait AChildNode {
    fn ws_node(&self) -> &web_sys::Node;

    fn append_to(&self, parent: &web_sys::Node) {
        parent
            .append_child(self.ws_node())
            .expect_throw("AChildNode::append_to");
    }

    fn insert_before_a_sibling(
        &self,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) {
        parent
            .insert_before(self.ws_node(), next_sibling)
            .expect_throw("AChildNode::insert_before");
    }

    fn remove_from(&self, parent: &web_sys::Node) {
        parent
            .remove_child(self.ws_node())
            .expect_throw("AChildNode::remove_from");
    }
}

impl AChildNode for web_sys::Node {
    fn ws_node(&self) -> &web_sys::Node {
        self
    }
}

pub trait MaybeAChildNode {
    fn ws_node(&self) -> Option<&web_sys::Node>;

    fn append_to(&self, parent: &web_sys::Node) {
        if let Some(node) = self.ws_node() {
            node.append_to(parent);
        }
    }

    fn insert_before_a_sibling(
        &self,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) {
        if let Some(node) = self.ws_node() {
            node.insert_before_a_sibling(parent, next_sibling);
        }
    }

    fn remove_from(&self, parent: &web_sys::Node) {
        if let Some(node) = self.ws_node() {
            node.remove_from(parent);
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ElementStatus {
    JustCreated,
    Existing,
    JustCloned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementType {
    Select,
    Input,
    Option,
    TextArea,
    Other,
}

impl From<&str> for ElementType {
    fn from(tag: &str) -> Self {
        match tag {
            "select" => Self::Select,
            "input" => Self::Input,
            "option" => Self::Option,
            "textarea" => Self::TextArea,
            _ => Self::Other,
        }
    }
}
