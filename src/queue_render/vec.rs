/*
element
    .qr_list_with_render(&self.qr_vec, spair::ListElementCreation::Clone, "div", |item, d| {});
*/
pub struct QrList<I> {
    values: Vec<I>,
    renders: Vec<Box<dyn std::any::Any>>,
}

pub enum ListDiff<I> {
    Push(I),
    Pop,
    Insert {
        // To support multi-change, we have to store a copy of the item here.
        index: usize,
        value: I,
    },
    RemoveAtIndex(usize),
}
