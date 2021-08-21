use spair::prelude::*;

const TODO_DATA_KEY: &str = "todos-data-for-spair";

pub(crate) fn write_data_to_storage(data: &super::TodoData) {
    let data = serde_json::to_string(&data).expect_throw("Unable to serialize todo items");
    spair::local_storage()
        .set_item(TODO_DATA_KEY, &data)
        .expect("Unable to set item on local storage")
}

pub(crate) fn read_data_from_storage() -> super::TodoData {
    let todo_items = spair::local_storage()
        .get_item(TODO_DATA_KEY)
        .expect_throw("Unable to get item from local storage");

    todo_items
        .map(|s| serde_json::from_str(&s).expect_throw("Unable to deserialize todo items"))
        .unwrap_or_default()
}
