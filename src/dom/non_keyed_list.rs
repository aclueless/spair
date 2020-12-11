#[must_use = "Caller should set selected option for <select> element"]
pub struct RememberSettingSelectedOption;

pub struct NonKeyedListUpdater<'a, C> {
    comp: &'a crate::component::Comp<C>,
    state: &'a C,
    tag: &'a str,
    use_template: bool,
    parent: &'a web_sys::Node,
    next_sibling: Option<&'a web_sys::Node>,
    list: &'a mut super::nodes::NodeList,
}

impl<'a, C: crate::component::Component> NonKeyedListUpdater<'a, C> {
    pub fn new(
        comp: &'a crate::component::Comp<C>,
        state: &'a C,
        list: &'a mut super::nodes::NodeList,
        tag: &'a str,
        parent: &'a web_sys::Node,
        next_sibling: Option<&'a web_sys::Node>,
        use_template: bool,
    ) -> Self {
        Self {
            comp,
            state,
            list,
            tag,
            use_template,
            parent,
            next_sibling,
        }
    }

    pub fn html_update<I, R>(
        &mut self,
        items: impl IntoIterator<Item = I>,
        render: R,
    ) -> RememberSettingSelectedOption
    where
        for<'i, 'c> R: Fn(&'i I, crate::dom::HtmlUpdater<'c, C>),
    {
        let mut index = 0;
        for item in items {
            let status = self.list.check_or_create_element_for_non_keyed_list(
                self.tag,
                index,
                self.parent,
                self.next_sibling,
                self.use_template,
            );
            let element = self.list.get_element(index);
            let u = super::ElementUpdater::new(self.comp, self.state, element, status);
            render(&item, u.into());
            index += 1;
        }
        self.clear_after(index);
        RememberSettingSelectedOption
    }

    pub fn html_update2<I, R>(
        &mut self,
        items: impl IntoIterator<Item = I>,
        render: R,
    ) -> RememberSettingSelectedOption
    where
        I: Copy,
        for<'u> R: Fn(I, crate::dom::HtmlUpdater<'u, C>),
    {
        let mut index = 0;
        for item in items {
            let status = self.list.check_or_create_element_for_non_keyed_list(
                self.tag,
                index,
                self.parent,
                self.next_sibling,
                self.use_template,
            );
            let element = self.list.get_element(index);
            let u = super::ElementUpdater::new(self.comp, self.state, element, status);
            render(item, u.into());
            index += 1;
        }
        self.clear_after(index);
        RememberSettingSelectedOption
    }

    fn clear_after(&mut self, index: usize) {
        if index >= self.list.count() {
            return;
        }
        if index == 0 && self.next_sibling.is_none() {
            self.parent.set_text_content(None);
            self.list.clear_raw();
        } else {
            self.list.clear_after(index, self.parent);
        }
    }

    #[cfg(feature = "svg")]
    pub fn svg_update<I, R>(&mut self, items: impl IntoIterator<Item = I>, render: R)
    where
        for<'i, 'c> R: Fn(&'i I, crate::dom::SvgUpdater<'c, C>),
    {
        let mut index = 0;
        for item in items {
            let status = self.list.check_or_create_svg_element_for_non_keyed_list(
                self.tag,
                index,
                self.parent,
                self.next_sibling,
                self.use_template,
            );
            let element = self.list.get_element(index);
            let u = super::ElementUpdater::new(self.comp, self.state, element, status);
            render(&item, u.into());
            index += 1;
        }
        self.clear_after(index);
    }
}
