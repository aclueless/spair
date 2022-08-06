use super::ElementRender;
use crate::component::{Comp, Component};
use crate::dom::{NameSpace, Nodes};

#[must_use = "Caller should set selected option for <select> element"]
pub struct RememberSettingSelectedOption;

pub struct ListRender<'a, C: Component> {
    comp: &'a Comp<C>,
    state: &'a C,
    tag: &'a str,
    use_template: bool,
    parent: &'a web_sys::Node,
    // This is None if it is a whole-list, the list is the only content of the parent node.
    // This is Some() if it is a partial-list, the parent contains the list and some other
    // nodes before or after the list.
    end_of_list_flag: Option<&'a web_sys::Node>,
    list: &'a mut Nodes,
}

impl<'a, C: Component> ListRender<'a, C> {
    pub fn new(
        comp: &'a Comp<C>,
        state: &'a C,
        list: &'a mut Nodes,
        tag: &'a str,
        parent: &'a web_sys::Node,
        end_of_list_flag: Option<&'a web_sys::Node>,
        use_template: bool,
    ) -> Self {
        Self {
            comp,
            state,
            tag,
            use_template,
            parent,
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

    pub fn render<N, I, II, R>(&mut self, items: II, render: R) -> RememberSettingSelectedOption
    where
        N: NameSpace,
        I: Copy,
        II: IntoIterator<Item = I>,
        for<'u> R: Fn(I, ElementRender<'u, C>),
    {
        let mut index = 0;
        for item in items {
            let status = self.list.check_or_create_element_for_list::<N>(
                self.tag,
                index,
                self.parent,
                self.end_of_list_flag,
                self.use_template,
            );
            let element = self.list.get_element_mut(index);
            let r = ElementRender::new(self.comp, self.state, element, status);
            render(item, r);
            index += 1;
        }
        self.clear_after(index);
        RememberSettingSelectedOption
    }
}
