use gloo_storage::{LocalStorage, Storage};
use spair::prelude::*;

const TODO_DATA_KEY: &str = "todos-data-for-spair";

pub(crate) fn write_data_to_storage(data: &super::TodoData) {
    LocalStorage::set(TODO_DATA_KEY, &data).expect_throw("Unable to set item on local storage")
}

pub(crate) fn read_data_from_storage() -> super::TodoData {
    LocalStorage::get(TODO_DATA_KEY).unwrap_or_default()
}
