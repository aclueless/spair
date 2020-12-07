pub trait FromElementUpdater<'a, C> {
    fn from(u: crate::dom::ElementUpdater<'a, C>) -> Self;
}

impl<'a, C> FromElementUpdater<'a, C> for crate::dom::HtmlUpdater<'a, C> {
    fn from(u: crate::dom::ElementUpdater<'a, C>) -> Self {
        From::from(u)
    }
}

impl<'a, C: crate::component::Component> super::KeyedListUpdater<'a, C> {
    pub fn update_with_render<I, G, K, R, U>(
        &mut self,
        items_state_iter: impl Iterator<Item = I> + DoubleEndedIterator,
        get_key: G,
        render: R,
    ) -> crate::dom::RememberSettingSelectedOption
    where
        for<'i> G: Fn(&'i I) -> K,
        K: 'a + Into<super::Key> + PartialEq<super::Key>,
        for<'i> R: Fn(&'i I, U),
        for<'c> U: FromElementUpdater<'c, C>,
    {
        // No items? Just clear the current list.
        if self.list_context.new_item_count == 0 {
            //self.remove_all_old_items();
            return crate::dom::RememberSettingSelectedOption;
        }

        // let mut items_state_iter = items_state_iter.peekable_double_ended();
        // if self.list_context.require_init_template {
        //     let u = super::ElementUpdater::new(
        //         self.comp,
        //         self.state,
        //         self.list_context.template.as_mut().unwrap(),
        //         super::ElementStatus::JustCreated,
        //     );

        //     items_state_iter.peek().unwrap_throw().render(u.into());
        // }
        // loop {
        //     let mut count = self.update_same_key_items_from_start(&mut items_state_iter);
        //     count += self.update_same_key_items_from_end(&mut items_state_iter);
        //     count += self.update_moved_forward_item(&mut items_state_iter);
        //     count += self.update_moved_backward_item(&mut items_state_iter);
        //     if count == 0 {
        //         break;
        //     }
        // }

        // self.update_other_items_in_middle(&mut items_state_iter);
        crate::dom::RememberSettingSelectedOption
    }
}
