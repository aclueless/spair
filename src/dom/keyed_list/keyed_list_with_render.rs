impl<'a, C: crate::component::Component> super::KeyedListUpdater<'a, C> {
    fn update_with_render<I, R, U>(
        &mut self,
        items_state_iter: impl Iterator<Item = I> + DoubleEndedIterator,
        render: R,
    ) -> crate::dom::RememberSettingSelectedOption
    where
        for<'i> R: Fn(&'i I, U),
        for<'c> U: From<crate::dom::ElementUpdater<'c, C>>,
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
