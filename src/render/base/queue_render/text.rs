use crate::{
    component::Component,
    dom::{ParentAndChild, QrNode, QrTextNode},
    render::base::NodesRender,
};

impl<'a, C: Component> NodesRender<'a, C> {
    // This method is called by incremental-dom, so it will return a new QrTextNode
    // on: New, or on Clone. If the node is an existing active node, it returns None.
    pub fn create_qr_text_node(&mut self) -> Option<QrTextNode<C>> {
        let tn = if self.new_node() {
            let tn = QrTextNode::new(self.comp());
            tn.insert_before(self.parent(), self.next_sibling());
            self.nodes_mut()
                .add_qr_node(QrNode::ActiveTextNode(Box::new(tn.clone())));
            Some(tn)
        } else {
            let comp = self.comp();
            let index = self.index();
            let qr_node = self.nodes_mut().get_qr_node(index);
            let rs = match qr_node {
                QrNode::ActiveTextNode(_) => None,
                QrNode::ClonedWsNode(wsn) => match wsn.take() {
                    Some(wsn) => {
                        let tn = QrTextNode::with_cloned_node(wsn, comp);
                        *qr_node = QrNode::ActiveTextNode(Box::new(tn.clone()));
                        Some(tn)
                    }
                    None => None,
                },
            };
            rs
        };
        self.next_index();
        tn
    }
}
