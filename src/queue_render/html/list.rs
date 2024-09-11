use wasm_bindgen::UnwrapThrowExt;

use crate::queue_render::vec::QrVec;

use crate::{
    component::Component,
    render::{
        base::{NodesUpdater, NodesUpdaterMut},
        html::{
            AttributesOnly, HtmlElementUpdater, HtmlNodesUpdater, Nodes, NodesOwned,
            StaticAttributes, StaticAttributesOnly,
        },
        ListElementCreation,
    },
};

pub trait HemsForQrList<'a, C: Component>: Sized + Into<NodesOwned<'a, C>> {
    fn qr_list<I, R>(self, list: &QrVec<I>, render: R)
    where
        I: 'static + Clone,
        R: 'static + Fn(I, crate::Nodes<C>),
    {
        qr_list(self, list, ListElementCreation::New, render)
    }

    fn qr_list_clone<I, R>(self, list: &QrVec<I>, render: R)
    where
        I: 'static + Clone,
        R: 'static + Fn(I, crate::Nodes<C>),
    {
        qr_list(self, list, ListElementCreation::Clone, render)
    }
}

fn qr_list<'a, C, T, I, R>(updater: T, list: &QrVec<I>, mode: ListElementCreation, render: R)
where
    I: 'static + Clone,
    R: 'static + Fn(I, crate::Nodes<C>),
    C: Component,
    T: Sized + Into<NodesOwned<'a, C>>,
{
    let mut nodes_updater: NodesOwned<C> = updater.into();
    let fn_render = move |entry: I, nodes: NodesUpdater<C>| {
        let mut nodes = HtmlNodesUpdater::new(nodes);
        render(entry, Nodes::new(&mut nodes));
    };
    let qr_list_render = match nodes_updater
        .nodes_updater_mut()
        .create_qr_list_render(true, mode, fn_render)
    {
        None => return,
        Some(render) => render,
    };
    list.content()
        .try_borrow_mut()
        .expect_throw("queue_render::html::list::HemsForQrList::qr_list content borrow mut")
        .add_render(Box::new(qr_list_render));
    list.check_and_queue_a_render();
}

impl<'a, C: Component> HemsForQrList<'a, C> for HtmlElementUpdater<'a, C> {}
impl<'a, C: Component> HemsForQrList<'a, C> for AttributesOnly<'a, C> {}
impl<'a, C: Component> HemsForQrList<'a, C> for StaticAttributes<'a, C> {}
impl<'a, C: Component> HemsForQrList<'a, C> for StaticAttributesOnly<'a, C> {}
