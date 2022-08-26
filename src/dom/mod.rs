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

pub enum ELementTag {
    Html(&'static str),
    #[cfg(feature = "svg")]
    Svg(&'static str),
}

impl ELementTag {
    pub fn namespace_and_tag(&self) -> (&str, &str) {
        match self {
            Self::Html(tag) => (crate::render::html::HtmlNameSpace::NAMESPACE, tag),
            #[cfg(feature = "svg")]
            Self::Svg(tag) => (crate::render::svg::SvgNameSpace::NAMESPACE, tag),
        }
    }
}

pub trait NameSpace {
    const NAMESPACE: &'static str;
}

#[deprecated = "Renamed this to AChildNode"]
pub trait ParentAndChild {
    fn ws_node(&self) -> &web_sys::Node;

    fn append_to(&self, parent: &web_sys::Node) {
        parent
            .append_child(self.ws_node())
            .expect_throw("ParentAndChild::append_to");
    }

    fn insert_before_a_sibling(
        &self,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) {
        parent
            .insert_before(self.ws_node(), next_sibling)
            .expect_throw("ParentAndChild::insert_before");
    }

    fn remove_from(&self, parent: &web_sys::Node) {
        parent
            .remove_child(self.ws_node())
            .expect_throw("ParentAndChild::remove_from");
    }
}

impl ParentAndChild for web_sys::Node {
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

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ElementStatus {
    JustCreated,
    Existing,
    JustCloned,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
