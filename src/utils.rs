use wasm_bindgen::UnwrapThrowExt;

pub fn window() -> web_sys::Window {
    web_sys::window().expect_throw("Unable to get window")
}

pub fn document() -> web_sys::Document {
    window().document().expect_throw("Unable to get document")
}

pub fn local_storage() -> web_sys::Storage {
    window()
        .local_storage()
        .expect_throw("Unable to access local storage")
        .expect_throw("No local storage found")
}

pub fn alert(message: &str) {
    window()
        .alert_with_message(message)
        .expect_throw("Error on displaying alert dialog");
}

pub fn confirm(message: &str) -> bool {
    window()
        .confirm_with_message(message)
        .expect_throw("Error on displaying confirm dialog")
}

pub fn prompt(message: &str, default_value: Option<&str>) -> Option<String> {
    match default_value {
        Some(default_value) => window()
            .prompt_with_message_and_default(message, default_value)
            .expect_throw("Error on getting user input with default value from the prompt dialog"),
        None => window()
            .prompt_with_message(message)
            .expect_throw("Error on getting user input from the prompt dialog"),
    }
}

pub(crate) fn register_event_listener_on_window(event: &str, listener: &js_sys::Function) {
    let window = crate::utils::window();
    let window: &web_sys::EventTarget = window.as_ref();
    window
        .add_event_listener_with_callback(event, listener)
        .expect_throw("Unable to register event listener on window");
}
