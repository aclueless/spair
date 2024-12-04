use spairc::{web_sys::MouseEvent, CallbackArg, Element, WsElement};

struct AppState {
    value: i32,
}

// fn updown_button(handler: Handler, text: &str) -> Button {
//     Button.on_click(handler).text(text)
// }

struct UpdownButtonGen {
    element: Element,
}

impl UpdownButtonGen {
    fn create(handler: CallbackArg<MouseEvent>, text: &str) -> Self {
        let mut element = Element::new("<button>?</button>", 1);
        element.click(0, handler);
        let node = element.ws_element().first_child();
        node.set_text_content(text);
        Self { element }
    }
}

// struct Counter {
//     value: i32,
// }

// #[render(state = Counter)]
// fn counter(root: Element<Counter>) {
//     let state = root.state();
//     let comp = root.comp();
//     Div.mount_to(root)
//         .child(Button.on_click(comp.decrease).text("-"))
//         .text(state.value)
//         .child(Button.on_click(comp.increase).text("+"))
//         .static_child(button(comp.decrease(), "-"))
//         .text(state.value)
//         .static_child(button(comp.decrease(), "-")).text("").are_you_here("").there_you_are("").good_morning("").over_here("");
// }

impl AppState {
    fn increase(&mut self) {
        self.value += 1;
    }

    fn decrease(&mut self) {
        self.value -= 1;
    }
}

struct Comp(spairc::Comp<AppState>);

impl Comp {
    pub fn increase(&self) -> CallbackArg<MouseEvent> {
        self.0.callback_arg(|state, _| state.increase())
    }

    pub fn decrease(&self) -> CallbackArg<MouseEvent> {
        self.0.callback_arg(|state, _| state.decrease())
    }
}

struct ComponentUpdater {
    element: Element,
    _db: UpdownButtonGen,
    _ub: UpdownButtonGen,
    current_value: WsElement,
    value_string: String,
}

impl spairc::Component for AppState {
    type Updater = ComponentUpdater;

    fn init(&self, comp: &spairc::Comp<Self>) -> (spairc::ComponentRoot, Self::Updater) {
        let comp = Comp(comp.clone());
        let db = UpdownButtonGen::create(comp.decrease(), "-");
        let ub = UpdownButtonGen::create(comp.increase(), "+");

        let element = Element::new("<div id='root'>???</div>", 0);
        let current_value = element.ws_element().first_child();

        element.insert_new_node_before_a_node(&db.element, Some(&current_value));
        element.append_new_node(ub.element.ws_element());
        let value_string = self.value.to_string();
        current_value.set_text_content(&value_string);

        element.replace_at_element_id("root");
        let updater = ComponentUpdater {
            element,
            _db: db,
            _ub: ub,
            current_value,
            value_string,
        };
        (
            spairc::ComponentRoot::Element(updater.element.ws_element().clone()),
            updater,
        )
    }

    fn render(&self, updater: &mut Self::Updater, _comp: &spairc::Comp<Self>) {
        updater
            .current_value
            .update_text_content_with_string(&mut updater.value_string, self.value.to_string())
    }
}

fn main() {
    // wasm_logger::init(wasm_logger::Config::default());
    spairc::start_app(AppState { value: 42 });
}
