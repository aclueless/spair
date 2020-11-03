use wasm_bindgen::UnwrapThrowExt;

#[derive(Default, Clone)]
pub struct NodeList(pub Vec<Node>);

impl std::fmt::Debug for NodeList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.write_fmt(format_args!("NodeList of {} items", self.0.len()))
    }
}

impl NodeList {
    pub fn count(&self) -> usize {
        self.0.len()
    }

    pub fn clear_raw(&mut self) {
        self.0.clear();
    }

    pub fn clear(&mut self, parent: &web_sys::Node) {
        self.0.drain(..).for_each(|mut node| node.clear(parent));
    }

    pub fn clear_after(&mut self, index: usize, parent: &web_sys::Node) {
        self.0
            .drain(index..)
            .for_each(|mut node| node.clear(parent));
    }

    pub fn append_to(&self, parent: &web_sys::Node) {
        self.0.iter().for_each(|node| node.append_to(parent));
    }

    pub fn get_element(&mut self, index: usize) -> &mut super::Element {
        match self
            .0
            .get_mut(index)
            .expect_throw("Expect an element node at the given index")
        {
            Node::Element(element) => element,
            _ => panic!("Why not an element?"),
        }
    }

    fn create_new_element(
        &mut self,
        tag: &str,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) {
        self.0.push(Node::Element(super::Element::new_in(
            tag,
            parent,
            next_sibling,
        )));
    }

    fn check_or_create_element(
        &mut self,
        tag: &str,
        index: usize,
        parent_status: super::ElementStatus,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) -> super::ElementStatus {
        if index == self.0.len() {
            self.create_new_element(tag, parent, next_sibling);
            super::ElementStatus::JustCreated
        } else {
            parent_status
        }
    }

    pub fn check_or_create_element_for_non_keyed_list(
        &mut self,
        tag: &str,
        index: usize,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
        use_template: bool,
    ) -> super::ElementStatus {
        let item_count = self.0.len();
        if index < item_count {
            super::ElementStatus::Existing
        } else if !use_template || item_count == 0 {
            self.create_new_element(tag, parent, next_sibling);
            super::ElementStatus::JustCreated
        } else {
            let element = self.0[0].clone();
            match &element {
                Node::Element(element) => element.insert_before(parent, next_sibling),
                _ => panic!("non-keyed-list: internal bug?"),
            }
            self.0.push(element);
            super::ElementStatus::JustCloned
        }
    }

    fn fragmented_node_list(
        &mut self,
        index: usize,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) -> &mut FragmentedNodeList {
        if index == self.0.len() {
            let fnl = FragmentedNodeList::default();
            parent
                .insert_before(fnl.end_node.as_ref(), next_sibling)
                .expect_throw("Unable to insert fragmented node into its parent node");
            self.0.push(Node::FragmentedNodeList(fnl));
        }

        match self
            .0
            .get_mut(index)
            .expect_throw("Expect a fragmented node list at the given index")
        {
            Node::FragmentedNodeList(fragmented_node_list) => fragmented_node_list,
            _ => panic!("Why not a fragmented_node_list?"),
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

    #[cfg(feature = "keyed-list")]
    pub fn keyed_list_context<'a>(
        &'a mut self,
        root_item_tag: &str,
        parent: &'a web_sys::Node,
        exact_count_of_new_items: usize,
        use_template: bool,
    ) -> super::KeyedListContext<'a> {
        if self.0.is_empty() {
            self.0.push(Node::KeyedList(super::KeyedList::default()));
        }

        match self
            .0
            .first_mut()
            .expect_throw("Expect a keyed list as the first item of the node list")
        {
            Node::KeyedList(list) => list.create_context(
                root_item_tag,
                exact_count_of_new_items,
                parent,
                use_template,
            ),
            _ => panic!("Why not a keyed list?"),
        }
    }

    pub fn store_component_handle(&mut self, any: AnyComponentHandle) {
        let any = Node::ComponentHandle(any);
        if let Some(first) = self.0.first_mut() {
            *first = any;
        } else {
            self.0.push(any);
        }
    }
}

#[derive(Clone)]
pub enum Node {
    Element(super::Element),
    Text(super::Text),
    FragmentedNodeList(FragmentedNodeList),
    #[cfg(feature = "keyed-list")]
    KeyedList(super::KeyedList),
    ComponentHandle(AnyComponentHandle),
}

impl Node {
    fn clear(&mut self, parent: &web_sys::Node) {
        match self {
            Self::Element(element) => element.remove_from(parent),
            Self::Text(text) => text.remove_from(parent),
            Self::FragmentedNodeList(mi) => mi.clear(parent),
            #[cfg(feature = "keyed-list")]
            Self::KeyedList(list) => list.clear(parent),
            Self::ComponentHandle(_) => {
                // The component is the only child of an element
                parent.set_text_content(None);
                // And the NodeList drop the ComponentHandle in its.clear()
            }
        }
    }

    fn append_to(&self, parent: &web_sys::Node) {
        match self {
            Self::Element(element) => element.append_to(parent),
            Self::Text(text) => text.append_to(parent),
            Self::FragmentedNodeList(mi) => mi.append_to(parent),
            #[cfg(feature = "keyed-list")]
            Self::KeyedList(list) => list.append_to(parent),
            Self::ComponentHandle(_) => {
                // TODO: Not sure what to do here???
                unreachable!("Node::ComponentHandle::append_to() is unreachable???");
            }
        }
    }
}

pub struct AnyComponentHandle(Box<dyn std::any::Any>);

impl Clone for AnyComponentHandle {
    fn clone(&self) -> Self {
        //
        panic!("Spair does not support mounting a component inside a list item");
    }
}

impl<C: crate::component::Component> From<crate::component::Comp<C>> for AnyComponentHandle {
    fn from(ch: crate::component::Comp<C>) -> Self {
        Self(Box::new(crate::component::ComponentHandle::from(ch)))
    }
}

// Manage a match-if arm or a partial-non-keyed-list
pub struct FragmentedNodeList {
    active_index: Option<usize>,
    // `end_node` marks the boundary of this fragment
    end_node: web_sys::Node,
    nodes: NodeList,
}

impl Clone for FragmentedNodeList {
    fn clone(&self) -> Self {
        // a FragmentedNodeList should not be cloned?
        Default::default()
    }
}

impl Default for FragmentedNodeList {
    fn default() -> Self {
        let end_node = crate::utils::document()
            .create_comment("Mark the end of a fragmented node list")
            .into();
        Self {
            active_index: None,
            end_node,
            nodes: NodeList::default(),
        }
    }
}

impl FragmentedNodeList {
    pub fn clear(&mut self, parent: &web_sys::Node) {
        self.nodes.clear(parent);
        parent
            .remove_child(&self.end_node)
            .expect_throw("Unable to remove FragmentedNodeList.end_node from its parent");
    }

    pub fn append_to(&self, parent: &web_sys::Node) {
        self.nodes.append_to(parent);
        parent.append_child(&self.end_node).expect_throw(
            "Unable to append a FragmentedNodeList's end node to its expected parent",
        );
    }
}

pub struct MatchIfUpdater<'a, C> {
    comp: &'a crate::component::Comp<C>,
    state: &'a C,

    parent: &'a web_sys::Node,
    match_if: &'a mut FragmentedNodeList,
}

impl<'a, C: crate::component::Component> MatchIfUpdater<'a, C> {
    pub fn render_on_arm_index(mut self, index: usize) -> super::NodesOwned<'a, C> {
        let status = if Some(index) != self.match_if.active_index {
            self.match_if.nodes.clear(self.parent.as_ref());
            self.match_if.active_index = Some(index);
            super::ElementStatus::JustCreated
        } else {
            super::ElementStatus::Existing
        };

        NodesOwned(NodeListUpdater {
            comp: self.comp,
            state: self.state,

            index: 0,
            parent_status: status,
            nodes: &mut self.match_if.nodes,
            parent: self.parent,
            next_sibling: Some(&self.match_if.end_node),
            #[cfg(feature = "partial-non-keyed-list")]
            select_element_value: super::SelectElementValue::none(),
        })
    }
}

pub(crate) struct NodeListUpdater<'a, C> {
    comp: &'a crate::component::Comp<C>,
    state: &'a C,

    index: usize,
    parent_status: super::ElementStatus,
    parent: &'a web_sys::Node,
    next_sibling: Option<&'a web_sys::Node>,
    nodes: &'a mut super::NodeList,
    #[cfg(feature = "partial-non-keyed-list")]
    select_element_value: super::SelectElementValue,
}

#[cfg(feature = "partial-non-keyed-list")]
impl<'a, C> Drop for NodeListUpdater<'a, C> {
    fn drop(&mut self) {
        self.select_element_value
            .set_select_element_value(self.parent);
    }
}

impl<'a, C: crate::component::Component> NodeListUpdater<'a, C> {
    pub fn from_el_updater(eu: super::ElementUpdater<'a, C>) -> Self {
        Self {
            comp: eu.comp,
            state: eu.state,
            index: 0,
            parent_status: eu.status,
            parent: eu.element.ws_element.as_ref(),
            next_sibling: None,
            nodes: &mut eu.element.nodes,
            #[cfg(feature = "partial-non-keyed-list")]
            select_element_value: eu.select_element_value,
        }
    }

    fn get_element_and_increase_index(&mut self, tag: &str) -> super::ElementUpdater<C> {
        let status = self.nodes.check_or_create_element(
            tag,
            self.index,
            self.parent_status,
            self.parent,
            self.next_sibling,
        );
        let element = self.nodes.get_element(self.index);
        self.index += 1;
        super::ElementUpdater::new(self.comp, self.state, element, status)
    }

    fn get_match_if_updater(&mut self) -> MatchIfUpdater<C> {
        let match_if = self
            .nodes
            .fragmented_node_list(self.index, self.parent, self.next_sibling);
        self.index += 1;
        MatchIfUpdater {
            comp: self.comp,
            state: self.state,
            parent: self.parent,
            match_if,
        }
    }

    #[cfg(feature = "partial-non-keyed-list")]
    pub fn list_with_render<I, R>(
        &mut self,
        items: impl IntoIterator<Item = I>,
        tag: &str,
        render: R,
        mode: super::ListElementCreation,
    ) where
        for<'i, 'c> R: Fn(&'i I, crate::Element<'c, C>),
    {
        let use_template = mode.use_template();
        let fragmented_node_list =
            self.nodes
                .fragmented_node_list(self.index, self.parent, self.next_sibling);
        self.index += 1;
        let mut non_keyed_list_updater = super::NonKeyedListUpdater::new(
            self.comp,
            self.state,
            &mut fragmented_node_list.nodes,
            tag,
            self.parent,
            Some(&fragmented_node_list.end_node),
            use_template,
        );
        let _select_element_value_will_be_set_on_dropping =
            non_keyed_list_updater.update(items, render);
    }
}

pub struct StaticNodesOwned<'a, C: crate::component::Component>(NodeListUpdater<'a, C>);

impl<'a, C: crate::component::Component> StaticNodesOwned<'a, C> {
    pub(super) fn from_el_updater(eu: super::ElementUpdater<'a, C>) -> Self {
        Self(NodeListUpdater::from_el_updater(eu))
    }

    pub fn state(&self) -> &'a C {
        self.0.state
    }

    pub fn comp(&self) -> crate::component::Comp<C> {
        self.0.comp.clone()
    }

    pub fn nodes(self) -> NodesOwned<'a, C> {
        NodesOwned(self.0)
    }

    /// Use this method when the compiler complains about expected `()` but found something else and you don't want to add a `;`
    pub fn done(self) {}

    pub fn render(mut self, value: impl crate::renderable::Render<C>) -> Self {
        let nodes = Nodes(&mut self.0);
        value.render(nodes);
        self
    }

    pub fn render_ref(mut self, value: &impl crate::renderable::RenderRef<C>) -> Self {
        let nodes = Nodes(&mut self.0);
        value.render(nodes);
        self
    }

    pub fn r#static(mut self, value: impl crate::renderable::StaticRender<C>) -> Self {
        let static_nodes = StaticNodes(&mut self.0);
        value.render(static_nodes);
        self
    }

    pub fn static_text_of_keyed_item(
        mut self,
        value: impl crate::renderable::ListItemStaticText<C>,
    ) -> Self {
        if self.0.parent_status != super::ElementStatus::Existing {
            value.render(self.nodes()).static_nodes()
        } else {
            self.0.index += 1;
            self
        }
    }
}

pub struct NodesOwned<'a, C: crate::component::Component>(NodeListUpdater<'a, C>);

impl<'a, C: crate::component::Component> NodesOwned<'a, C> {
    pub(super) fn from_el_updater(eu: super::ElementUpdater<'a, C>) -> Self {
        Self(NodeListUpdater::from_el_updater(eu))
    }

    pub(super) fn nodes_ref<'n>(&'n mut self) -> Nodes<'n, 'a, C> {
        Nodes(&mut self.0)
    }

    pub(super) fn static_nodes_ref<'n>(&'n mut self) -> StaticNodes<'n, 'a, C> {
        StaticNodes(&mut self.0)
    }

    pub fn state(&self) -> &'a C {
        self.0.state
    }

    pub fn comp(&self) -> crate::component::Comp<C> {
        self.0.comp.clone()
    }

    pub fn static_nodes(self) -> StaticNodesOwned<'a, C> {
        StaticNodesOwned(self.0)
    }

    /// Use this method when the compiler complains about expected `()` but found something else and you don't want to add a `;`
    pub fn done(self) {}

    pub fn render(mut self, value: impl crate::renderable::Render<C>) -> Self {
        let nodes = Nodes(&mut self.0);
        value.render(nodes);
        self
    }

    pub fn render_ref(mut self, value: &impl crate::renderable::RenderRef<C>) -> Self {
        let nodes = Nodes(&mut self.0);
        value.render(nodes);
        self
    }

    pub fn r#static(mut self, value: impl crate::renderable::StaticRender<C>) -> Self {
        let static_nodes = StaticNodes(&mut self.0);
        value.render(static_nodes);
        self
    }

    pub fn static_text_of_keyed_item(
        mut self,
        value: impl crate::renderable::ListItemStaticText<C>,
    ) -> Self {
        if self.0.parent_status != super::ElementStatus::Existing {
            value.render(self)
        } else {
            self.0.index += 1;
            self
        }
    }

    pub(crate) fn update_text(mut self, text: &str) -> Self {
        self.0
            .nodes
            .update_text(self.0.index, text, self.0.parent, self.0.next_sibling);
        self.0.index += 1;
        self
    }

    #[cfg(feature = "partial-non-keyed-list")]
    pub fn list_with_render<I, R>(
        mut self,
        items: impl IntoIterator<Item = I>,
        tag: &str,
        render: R,
        mode: super::ListElementCreation,
    ) -> Self
    where
        for<'i, 'c> R: Fn(&'i I, crate::Element<'c, C>),
    {
        self.0.list_with_render(items, tag, render, mode);
        self
    }
}

macro_rules! create_methods_for_tags {
    ($($tag:ident)+) => {
        $(
            fn $tag(self, f: impl FnOnce(super::ElementUpdater<C>)) -> Self::Output {
                self.render_element(stringify!($tag), f)
            }
        )+
    }
}

mod sealed {
    pub trait DomBuilder<C: crate::component::Component> {
        fn require_render(&self) -> bool;
        fn just_created(&self) -> bool;
        fn next_index(&mut self);
        fn get_element_and_increase_index(&mut self, tag: &str) -> crate::dom::ElementUpdater<C>;
        fn get_match_if_and_increase_index(&mut self) -> super::MatchIfUpdater<C>;
        fn store_raw_wrapper(&mut self, element: crate::dom::Element);
    }
}

pub trait DomBuilder<C: crate::component::Component>: Sized {
    type Output: From<Self> + sealed::DomBuilder<C>;

    fn match_if(self, f: impl FnOnce(MatchIfUpdater<C>)) -> Self::Output {
        use sealed::DomBuilder;
        let mut this: Self::Output = self.into();
        f(this.get_match_if_and_increase_index());
        this
    }

    fn render_element(self, tag: &str, f: impl FnOnce(super::ElementUpdater<C>)) -> Self::Output {
        use sealed::DomBuilder;
        let mut this: Self::Output = self.into();
        if this.require_render() {
            f(this.get_element_and_increase_index(tag));
        } else {
            this.next_index();
        }
        this
    }

    create_methods_for_tags! {
        a abbr address area article aside audio
        b bdi bdo blockquote button br
        canvas caption cite code col colgroup
        data datalist dd del details dfn dialog div dl dt
        em embed
        fieldset figcaption figure footer form
        h1 h2 h3 h4 h5 h6 header hgroup hr
        i iframe img input ins
        kbd
        label legend li
        main map mark menu meter
        nav
        object ol optgroup option output
        p param picture pre progress
        q
        rp rt ruby
        s samp section select slot small source span strong sub summary sup
        table tbody td template textarea tfoot th thead time tr track
        u ul
        var video
        wbr //should be specialized?
    }

    fn line_break(self) -> Self::Output {
        use sealed::DomBuilder;
        let mut this: Self::Output = self.into();
        if this.require_render() {
            this.get_element_and_increase_index("br");
        } else {
            this.next_index();
        }
        this
    }

    fn horizontal_line(self) -> Self::Output {
        use sealed::DomBuilder;
        let mut this: Self::Output = self.into();
        if this.require_render() {
            this.get_element_and_increase_index("hr");
        } else {
            this.next_index();
        }
        this
    }

    fn raw_wrapper(self, raw_wrapper: &impl super::RawWrapper<C>) -> Self::Output {
        use sealed::DomBuilder;
        let mut this: Self::Output = self.into();
        if this.just_created() {
            let ws_element = raw_wrapper.ws_element();
            // TODO: should raw element stores in its own variant?
            //      This store the ws_element of the RawWrapper as a super::Element,
            //      it may cause a problem when the RawWrapper in side a list element
            let element = super::Element::from_ws_element(ws_element.clone());
            this.store_raw_wrapper(element);
            raw_wrapper.mounted();
        }
        this.next_index();

        this
    }
}

impl<'a, C: crate::component::Component> sealed::DomBuilder<C> for StaticNodesOwned<'a, C> {
    fn require_render(&self) -> bool {
        self.0.parent_status == super::ElementStatus::JustCreated
    }

    fn just_created(&self) -> bool {
        self.0.parent_status == super::ElementStatus::JustCreated
    }

    fn next_index(&mut self) {
        self.0.index += 1;
    }

    fn get_element_and_increase_index(&mut self, tag: &str) -> super::ElementUpdater<C> {
        self.0.get_element_and_increase_index(tag)
    }

    fn get_match_if_and_increase_index(&mut self) -> MatchIfUpdater<C> {
        self.0.get_match_if_updater()
    }

    fn store_raw_wrapper(&mut self, element: super::Element) {
        element.insert_before(self.0.parent, self.0.next_sibling);
        self.0.nodes.0.push(Node::Element(element));
    }
}

impl<'a, C: crate::component::Component> DomBuilder<C> for StaticNodesOwned<'a, C> {
    type Output = Self;
}

impl<'a, C: crate::component::Component> sealed::DomBuilder<C> for NodesOwned<'a, C> {
    fn require_render(&self) -> bool {
        true
    }

    fn just_created(&self) -> bool {
        self.0.parent_status == super::ElementStatus::JustCreated
    }

    fn next_index(&mut self) {
        self.0.index += 1;
    }

    fn get_element_and_increase_index(&mut self, tag: &str) -> super::ElementUpdater<C> {
        self.0.get_element_and_increase_index(tag)
    }

    fn get_match_if_and_increase_index(&mut self) -> MatchIfUpdater<C> {
        self.0.get_match_if_updater()
    }

    fn store_raw_wrapper(&mut self, element: super::Element) {
        element.insert_before(self.0.parent, self.0.next_sibling);
        self.0.nodes.0.push(Node::Element(element));
    }
}

impl<'a, C: crate::component::Component> DomBuilder<C> for NodesOwned<'a, C> {
    type Output = Self;
}

pub struct StaticNodes<'n, 'h: 'n, C: crate::component::Component>(&'n mut NodeListUpdater<'h, C>);

impl<'n, 'h, C: crate::component::Component> StaticNodes<'n, 'h, C> {
    pub fn state(&self) -> &'n C {
        self.0.state
    }

    pub fn comp(&self) -> crate::component::Comp<C> {
        self.0.comp.clone()
    }

    pub fn nodes(self) -> Nodes<'n, 'h, C> {
        Nodes(self.0)
    }

    pub fn render(self, value: impl crate::renderable::Render<C>) -> Self {
        let nodes = Nodes(self.0);
        value.render(nodes);
        self
    }

    pub fn render_ref(self, value: &impl crate::renderable::RenderRef<C>) -> Self {
        let nodes = Nodes(self.0);
        value.render(nodes);
        self
    }

    pub fn r#static(self, value: impl crate::renderable::StaticRender<C>) -> Self {
        let static_nodes = StaticNodes(self.0);
        value.render(static_nodes);
        self
    }

    // pub fn static_text_of_keyed_item(
    //     mut self,
    //     value: impl crate::renderable::ListItemStaticText<C>,
    // ) -> Self {
    //     if self.0.parent_status != super::ElementStatus::Existing {
    //         value.render(self.nodes()).static_nodes()
    //     } else {
    //         self.0.index += 1;
    //         self
    //     }
    // }

    pub(crate) fn static_text(self, text: &str) -> Self {
        self.0
            .nodes
            .static_text(self.0.index, text, self.0.parent, self.0.next_sibling);
        self.0.index += 1;
        self
    }
}

pub struct Nodes<'n, 'h: 'n, C: crate::component::Component>(&'n mut NodeListUpdater<'h, C>);

impl<'n, 'h, C: crate::component::Component> Nodes<'n, 'h, C> {
    pub fn state(&self) -> &'n C {
        self.0.state
    }

    pub fn comp(&self) -> crate::component::Comp<C> {
        self.0.comp.clone()
    }

    pub fn static_nodes(self) -> StaticNodes<'n, 'h, C> {
        StaticNodes(self.0)
    }

    pub fn render(self, value: impl crate::renderable::Render<C>) -> Self {
        let nodes = Nodes(self.0);
        value.render(nodes);
        self
    }

    pub fn render_ref(self, value: &impl crate::renderable::RenderRef<C>) -> Self {
        let nodes = Nodes(self.0);
        value.render(nodes);
        self
    }

    pub fn r#static(self, value: impl crate::renderable::StaticRender<C>) -> Self {
        let static_nodes = StaticNodes(self.0);
        value.render(static_nodes);
        self
    }

    // pub fn static_text_of_keyed_item(
    //     mut self,
    //     value: impl crate::renderable::ListItemStaticText<C>,
    // ) -> Self {
    //     if self.0.parent_status != super::ElementStatus::Existing {
    //         value.render(self)
    //     } else {
    //         self.0.index += 1;
    //         self
    //     }
    // }

    pub(crate) fn update_text(self, text: &str) -> Self {
        self.0
            .nodes
            .update_text(self.0.index, text, self.0.parent, self.0.next_sibling);
        self.0.index += 1;
        self
    }

    #[cfg(feature = "partial-non-keyed-list")]
    pub fn list_with_render<I, R>(
        self,
        items: impl IntoIterator<Item = I>,
        tag: &str,
        render: R,
        mode: super::ListElementCreation,
    ) -> Self
    where
        for<'i, 'c> R: Fn(&'i I, crate::Element<'c, C>),
    {
        self.0.list_with_render(items, tag, render, mode);
        self
    }
}

impl<'n, 'h, C: crate::component::Component> sealed::DomBuilder<C> for StaticNodes<'n, 'h, C> {
    fn require_render(&self) -> bool {
        self.0.parent_status == super::ElementStatus::JustCreated
    }

    fn just_created(&self) -> bool {
        self.0.parent_status == super::ElementStatus::JustCreated
    }

    fn next_index(&mut self) {
        self.0.index += 1;
    }

    fn get_element_and_increase_index(&mut self, tag: &str) -> super::ElementUpdater<C> {
        self.0.get_element_and_increase_index(tag)
    }

    fn get_match_if_and_increase_index(&mut self) -> MatchIfUpdater<C> {
        self.0.get_match_if_updater()
    }

    fn store_raw_wrapper(&mut self, element: super::Element) {
        element.insert_before(self.0.parent, self.0.next_sibling);
        self.0.nodes.0.push(Node::Element(element));
    }
}

impl<'n, 'h, C: crate::component::Component> DomBuilder<C> for StaticNodes<'n, 'h, C> {
    type Output = Self;
}

impl<'n, 'h, C: crate::component::Component> sealed::DomBuilder<C> for Nodes<'n, 'h, C> {
    fn require_render(&self) -> bool {
        true
    }

    fn just_created(&self) -> bool {
        self.0.parent_status == super::ElementStatus::JustCreated
    }

    fn next_index(&mut self) {
        self.0.index += 1;
    }

    fn get_element_and_increase_index(&mut self, tag: &str) -> super::ElementUpdater<C> {
        self.0.get_element_and_increase_index(tag)
    }

    fn get_match_if_and_increase_index(&mut self) -> MatchIfUpdater<C> {
        self.0.get_match_if_updater()
    }

    fn store_raw_wrapper(&mut self, element: super::Element) {
        element.insert_before(self.0.parent, self.0.next_sibling);
        self.0.nodes.0.push(Node::Element(element));
    }
}

impl<'n, 'h, C: crate::component::Component> DomBuilder<C> for Nodes<'n, 'h, C> {
    type Output = Self;
}

impl<'a, C: crate::component::Component> From<super::ElementUpdater<'a, C>> for NodesOwned<'a, C> {
    fn from(eu: super::ElementUpdater<'a, C>) -> Self {
        Self::from_el_updater(eu)
    }
}

impl<'a, C: crate::component::Component> DomBuilder<C> for super::ElementUpdater<'a, C> {
    type Output = NodesOwned<'a, C>;
}

impl<'a, C: crate::component::Component> From<super::StaticAttributes<'a, C>>
    for NodesOwned<'a, C>
{
    fn from(sa: super::StaticAttributes<'a, C>) -> Self {
        sa.nodes()
    }
}

impl<'a, C: crate::component::Component> DomBuilder<C> for super::StaticAttributes<'a, C> {
    type Output = NodesOwned<'a, C>;
}
