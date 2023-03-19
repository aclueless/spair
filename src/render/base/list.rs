use super::NodesUpdater;
use crate::{
    component::{Comp, Component},
    dom::{ElementStatus, Nodes},
};

#[must_use = "Caller should set selected option for <select> element"]
pub struct RememberSettingSelectedOption;

pub struct ListUpdater<'a> {
    use_template: bool,
    parent: &'a web_sys::Node,
    parent_status: ElementStatus,
    // This is None if it is a whole-list, the list is the only content of the parent node.
    // This is Some() if it is a partial-list, the parent contains the list and some other
    // nodes before or after the list.
    end_of_list_flag: Option<&'a web_sys::Node>,
    list: &'a mut Nodes,
}

impl<'a> ListUpdater<'a> {
    pub fn new(
        list: &'a mut Nodes,
        parent: &'a web_sys::Node,
        parent_status: ElementStatus,
        end_of_list_flag: Option<&'a web_sys::Node>,
        use_template: bool,
    ) -> Self {
        Self {
            use_template,
            parent,
            parent_status,
            end_of_list_flag,
            list,
        }
    }

    pub fn clear_after(&mut self, index: usize) {
        if index >= self.list.count() {
            return;
        }
        if index == 0 && self.end_of_list_flag.is_none() {
            self.parent.set_text_content(None);
            self.list.clear_vec();
        } else {
            self.list.remove_from_dom_after(index, self.parent);
        }
    }

    pub fn render<C, I, II, R>(
        &mut self,
        comp: &Comp<C>,
        state: &C,
        items: II,
        render: R,
    ) -> RememberSettingSelectedOption
    where
        C: Component,
        II: Iterator<Item = I>,
        R: Fn(I, NodesUpdater<C>),
    {
        let mut index = 0;
        for item in items {
            let (status, group, next_sibling) = self.list.recipe_for_list_entry(
                index,
                self.parent,
                self.parent_status,
                self.end_of_list_flag,
                self.use_template,
            );
            let u = NodesUpdater::new(
                comp,
                state,
                status,
                self.parent,
                next_sibling.as_ref().or(self.end_of_list_flag),
                group.nodes_mut(),
            );
            render(item, u);
            index += 1;
        }
        self.clear_after(index);
        RememberSettingSelectedOption
    }
}
