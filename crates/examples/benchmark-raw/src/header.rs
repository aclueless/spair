use spairc::{web_sys::MouseEvent, CallbackArg, Context, Element};

use crate::AppState;

pub struct ButtonViewState {
    root_element: Element,
    _button_element: Element,
}

impl ButtonViewState {
    pub fn create(id: &str, text: &str, callback: CallbackArg<MouseEvent>) -> Self {
        const HTML:&str = "<div class='col-sm-6 smallpad'><button type='button' class='btn btn-primary btn-block'></button></div>";
        let root_element = Element::with_html(HTML, 0);
        let mut button_element = root_element.first_child().create_element_with_capacity(1);
        // let mut button_element = button_element.first_child().create_element_with_capacity(1);
        button_element.set_id(id);
        button_element.set_text_content(text);
        button_element.click(0, callback);

        ButtonViewState {
            root_element,
            _button_element: button_element,
        }
    }
}

pub struct HeaderViewState {
    pub root_element: Element,
    _run: ButtonViewState,
    _runlots: ButtonViewState,
    _add: ButtonViewState,
    _update: ButtonViewState,
    _clear: ButtonViewState,
    _swaprows: ButtonViewState,
}

impl HeaderViewState {
    pub fn create(context: &Context<AppState>) -> Self {
        const HTML: &str = "<div class='jumbotron'><div class='row'><div class='col-md-6'><h1>Spair keyed</h1></div><div class='col-md-6'><div class='row'></div></div></div></div>";
        let root_element = Element::with_html(HTML, 0);

        let run = ButtonViewState::create(
            "run",
            "Create 1,000 rows",
            context.comp.callback_arg(|state, _| state.create(1000)),
        );
        let runlots = ButtonViewState::create(
            "runlots",
            "Create 10,000 rows",
            context.comp.callback_arg(|state, _| state.create(10000)),
        );
        let add = ButtonViewState::create(
            "add",
            "Append 1,000 rows",
            context.comp.callback_arg(|state, _| state.append(1000)),
        );
        let update = ButtonViewState::create(
            "update",
            "Update every 10th row",
            context.comp.callback_arg(|state, _| state.update()),
        );
        let clear = ButtonViewState::create(
            "clear",
            "Clear",
            context.comp.callback_arg(|state, _| state.clear()),
        );
        let swaprows = ButtonViewState::create(
            "swaprows",
            "Swap Rows",
            context.comp.callback_arg(|state, _| state.swap()),
        );
        let buttons = root_element
            .first_child()
            .first_child()
            .next_sibling()
            .first_child();
        buttons.insert_new_node_before_a_node(&run.root_element, None);
        buttons.insert_new_node_before_a_node(&runlots.root_element, None);
        buttons.insert_new_node_before_a_node(&add.root_element, None);
        buttons.insert_new_node_before_a_node(&update.root_element, None);
        buttons.insert_new_node_before_a_node(&clear.root_element, None);
        buttons.insert_new_node_before_a_node(&swaprows.root_element, None);
        HeaderViewState {
            root_element,
            _run: run,
            _runlots: runlots,
            _add: add,
            _update: update,
            _clear: clear,
            _swaprows: swaprows,
        }
    }
}
