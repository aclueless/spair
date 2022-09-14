use super::ElementRender;
use crate::{
    component::{Comp, Component},
    dom::{ElementStatus, ElementTag, Nodes},
};

#[must_use = "Caller should set selected option for <select> element"]
pub struct RememberSettingSelectedOption;

pub struct ListRender<'a> {
    use_template: bool,
    parent: &'a web_sys::Node,
    parent_status: ElementStatus,
    // This is None if it is a whole-list, the list is the only content of the parent node.
    // This is Some() if it is a partial-list, the parent contains the list and some other
    // nodes before or after the list.
    end_of_list_flag: Option<&'a web_sys::Node>,
    list: &'a mut Nodes,
}

impl<'a> ListRender<'a> {
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
            self.list.clear_after(index, self.parent);
        }
    }

    pub fn render<C, E, I, II, R>(
        &mut self,
        comp: &Comp<C>,
        state: &C,
        items: II,
        tag: E,
        render: R,
    ) -> RememberSettingSelectedOption
    where
        C: Component,
        E: ElementTag,
        II: Iterator<Item = I>,
        for<'r> R: Fn(&I, ElementRender<'r, C>),
    {
        let mut index = 0;
        for item in items {
            let status = self.list.check_or_create_element_for_list(
                tag,
                index,
                self.parent,
                self.parent_status,
                self.end_of_list_flag,
                self.use_template,
            );
            let element = self.list.get_element_mut(index);
            let r = ElementRender::new(comp, state, element, status);
            render(&item, r);
            index += 1;
        }
        self.clear_after(index);
        RememberSettingSelectedOption
    }
}
