use spair::prelude::*;

const TODO_ITEMS_KEY: &'static str = "spair-todo-items";

pub fn write_items_to_storage(items: &Vec<super::TodoItem>) {
    let data = serde_json::to_string(&items).expect_throw("Unable to serialize todo items");
    get_local_storage()
        .set_item(TODO_ITEMS_KEY, &data)
        .expect("Unable to set item on local storage")
}

pub fn read_items_from_storage() -> Vec<super::TodoItem> {
    let todo_items = get_local_storage()
        .get_item(TODO_ITEMS_KEY)
        .expect_throw("Unable to get item from local storage");

    todo_items
        .map(|s| serde_json::from_str(&s).expect_throw("Unable to deserialize todo items"))
        .unwrap_or(Vec::new())
}

fn get_local_storage() -> web_sys::Storage {
    spair::window()
        .local_storage()
        .expect_throw("Unable to access local storage")
        .expect_throw("No local storage found")
}
