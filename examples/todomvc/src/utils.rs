use spair::prelude::*;

const TODO_DATA_KEY: &'static str = "spair-todo-data";

fn get_local_storage() -> web_sys::Storage {
    spair::window()
        .local_storage()
        .expect_throw("Unable to access local storage")
        .expect_throw("No local storage found")
}

pub(crate) fn write_data_to_storage(data: &super::TodoData) {
    let data = serde_json::to_string(&data).expect_throw("Unable to serialize todo items");
    get_local_storage()
        .set_item(TODO_DATA_KEY, &data)
        .expect("Unable to set item on local storage")
}

pub(crate) fn read_data_from_storage() -> super::TodoData {
    let todo_items = get_local_storage()
        .get_item(TODO_DATA_KEY)
        .expect_throw("Unable to get item from local storage");

    todo_items
        .map(|s| serde_json::from_str(&s).expect_throw("Unable to deserialize todo items"))
        .unwrap_or(super::TodoData::default())
}
