use crate::{
    component::Component,
    dom::{AChildNode, ELementTag},
    queue_render::dom::{QrList, QrNode, QrTextNode},
    render::{
        base::{ElementRender, NodesRender},
        ListElementCreation,
    },
};

impl<'a, C: Component> NodesRender<'a, C> {
    // This method is called by incremental-dom, so it will return a new QrTextNode
    // on: New, or on Clone. If the node is an existing active node, it returns None.
    pub fn create_qr_text_node(&mut self) -> Option<QrTextNode> {
        let tn = if self.new_node() {
            let tn = QrTextNode::new();
            tn.insert_before_a_sibling(self.parent(), self.next_sibling());
            self.nodes_mut().add_qr_node(QrNode::Text(tn.clone()));
            Some(tn)
        } else {
            let index = self.index();
            let qr_node = self.nodes_mut().get_qr_node(index);
            match qr_node {
                QrNode::Text(_) => None,
                QrNode::ClonedWsNode(wsn) => match wsn.take() {
                    Some(wsn) => {
                        let tn = QrTextNode::with_cloned_node(wsn);
                        *qr_node = QrNode::Text(tn.clone());
                        Some(tn)
                    }
                    None => None,
                },
                QrNode::List(_) => {
                    panic!("spair internal error: Expect a ClonedWsNode or Text");
                }
            }
        };
        self.next_index();
        tn
    }

    pub fn create_qr_list<I, R>(
        &mut self,
        mode: ListElementCreation,
        tag: ELementTag,
        fn_render: R,
        full_list: bool,
    ) -> Option<QrList<C, I>>
    where
        for<'r> R: 'static + Fn(&I, ElementRender<'r, C>),
    {
        let list = if self.new_node() {
            let end_flag_node = if full_list {
                None
            } else {
                let n: web_sys::Node = crate::utils::document()
                    .create_comment("Mark the end of a qr list")
                    .into();
                n.insert_before_a_sibling(self.parent(), self.next_sibling());
                Some(n)
            };

            let list = QrList::new(
                tag,
                self.comp(),
                self.parent().clone(),
                end_flag_node,
                fn_render,
                mode.use_template(),
            );
            self.nodes_mut()
                .add_qr_node(QrNode::List(list.make_representative()));
            Some(list)
        } else {
            let index = self.index();
            let comp = self.comp();
            let parent = self.parent().clone();
            let qr_node = self.nodes_mut().get_qr_node(index);
            match qr_node {
                QrNode::List(_) => None,
                QrNode::ClonedWsNode(wsn) => match wsn.take() {
                    Some(wsn) => {
                        let list = QrList::new(
                            tag,
                            comp,
                            parent,
                            Some(wsn),
                            fn_render,
                            mode.use_template(),
                        );
                        *qr_node = QrNode::List(list.make_representative());
                        Some(list)
                    }
                    None => None,
                },
                QrNode::Text(_) => {
                    panic!("spair internal error: Expect a ClonedWsNode or Text");
                }
            }
        };
        self.next_index();
        list
    }
}
