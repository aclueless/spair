//use wasm_bindgen::UnwrapThrowExt;

/*
#[derive(Default)]
pub struct NonKeyedList {
    elements: Vec<super::Element>,

    // None => a full list: the parent node only has child elements that are in this list
    // Some => partial list: the parent node also has some child elements that are not in this list
    end_node: Option<web_sys::Node>,
}

impl Clone for NonKeyedList {
    fn clone(&self) -> Self {
        // No clone for non keyed list
        Self::new()
    }
}

impl NonKeyedList {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
            end_node: None,
        }
    }

    #[cfg(feature = "partial-non-keyed-list")]
    pub fn create_end_node(
        mut self,
        parent: &web_sys::Node,
        next_sibling: Option<&web_sys::Node>,
    ) -> Self {
        let end_node = crate::utils::document()
            .create_comment("Mark the end of a partial list")
            .into();
        parent
            .insert_before(&end_node, next_sibling)
            .expect_throw("Unable to insert a child Element to its expected parent");
        self.end_node = Some(end_node);
        self
    }

    pub fn clear(&mut self, parent: &web_sys::Node) {
        self.elements
            .drain(..)
            .for_each(|element| element.remove_from(parent));
        if let Some(end_node) = self.end_node.as_ref() {
            parent
                .remove_child(end_node)
                .expect_throw("Unable to remove NonKeyedList.end_node from its parent");
        }
    }

    pub fn append_to(&self, parent: &web_sys::Node) {
        self.elements
            .iter()
            .for_each(|element| element.append_to(parent));
        if let Some(end_node) = self.end_node.as_ref() {
            parent
                .append_child(end_node)
                .expect_throw("Unable to append a match/if arm's end node to its expected parent");
        }
    }

    pub fn create_element(
        &mut self,
        tag: &str,
        index: usize,
        parent: &web_sys::Node,
        use_template: bool,
    ) -> super::ElementStatus {
        let item_count = self.elements.len();
        if index < item_count {
            super::ElementStatus::Existing
        } else if !use_template || item_count == 0 {
            self.elements
                .push(super::Element::new_in(tag, parent, self.end_node.as_ref()));
            super::ElementStatus::JustCreated
        } else {
            let element = self.elements[0].clone();
            element.insert_before(parent, self.end_node.as_ref());
            self.elements.push(element);
            super::ElementStatus::JustCloned
        }
    }

    pub fn get_element(&mut self, index: usize) -> &mut super::Element {
        self.elements
            .get_mut(index)
            .expect_throw("Expect an element node at the given index")
    }

    pub fn clear_after(&mut self, index: usize, parent: &web_sys::Node) {
        if index < self.elements.len() {
            if index == 0 && self.end_node.is_none() {
                parent.set_text_content(None);
                self.elements.clear();
            } else {
                self.elements
                    .drain(index..)
                    .for_each(|element| element.remove_from(parent));
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[must_use = "Caller should set selected option for <select> element"]
    pub fn update<I, R, C>(
        &mut self,
        items: impl IntoIterator<Item = I>,
        tag: &str,
        render: R,
        use_template: bool,
        parent: &web_sys::Node,
        comp: &crate::component::Comp<C>,
        state: &C,
    ) -> RememberSettingSelectedOption
    where
        C: crate::component::Component,
        for<'i, 'c> R: Fn(&'i I, super::ElementUpdater<'c, C>),
    {
        let mut index = 0;
        for item in items {
            let status = self.create_element(tag, index, parent, use_template);
            let element = self.get_element(index);
            let eu = super::ElementUpdater::new(comp, state, element, status);
            render(&item, eu);
            index += 1;
        }
        self.clear_after(index, parent);
        RememberSettingSelectedOption
    }
}
*/

#[must_use = "Caller should set selected option for <select> element"]
pub struct RememberSettingSelectedOption;

pub struct NonKeyedListUpdater<'a, C> {
    comp: &'a crate::component::Comp<C>,
    state: &'a C,
    tag: &'a str,
    use_template: bool,
    parent: &'a web_sys::Node,
    next_sibling: Option<&'a web_sys::Node>,
    list: &'a mut super::NodeList,
}

impl<'a, C: crate::component::Component> NonKeyedListUpdater<'a, C> {
    pub fn new(
        comp: &'a crate::component::Comp<C>,
        state: &'a C,
        list: &'a mut super::NodeList,
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

    pub fn update<I, R>(
        &mut self,
        items: impl IntoIterator<Item = I>,
        render: R,
    ) -> RememberSettingSelectedOption
    where
        for<'i, 'c> R: Fn(&'i I, super::ElementUpdater<'c, C>),
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
            let eu = super::ElementUpdater::new(self.comp, self.state, element, status);
            render(&item, eu);
            index += 1;
        }
        self.clear_after(index);
        RememberSettingSelectedOption
    }

    pub fn clear_after(&mut self, index: usize) {
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
}
