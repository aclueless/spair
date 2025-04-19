use wasm_bindgen::UnwrapThrowExt;
use web_sys::Node;

use crate::{
    CallbackArg, Context, Element, WsElement,
    component::{RcComp, create_component},
    helper,
};

pub trait TestDataInterface: Sized + 'static {
    type ViewState;
    fn init(&self, root: &Element, context: &Context<TestComp<Self>>) -> Self::ViewState;
    fn update(&self, updater: &mut Self::ViewState, context: &Context<TestComp<Self>>);
}
pub struct TestComp<T> {
    data: T,
}

pub struct CompUpdater<T: TestDataInterface> {
    _root: Element,
    test_updater: T::ViewState,
}

impl<T: 'static + TestDataInterface> crate::Component for TestComp<T> {
    type ViewState = CompUpdater<T>;

    fn create_view(ccontext: &Context<Self>) -> (WsElement, Self::ViewState) {
        let element = Element::with_html("<div id='spair_test'></div>", 0);
        let test_updater = ccontext.state.data.init(&element, ccontext);
        let body = element.append_to_body();
        (
            body,
            CompUpdater {
                _root: element,
                test_updater,
            },
        )
    }

    fn update_view(updater: &mut Self::ViewState, ucontext: &crate::Context<Self>) {
        ucontext
            .state
            .data
            .update(&mut updater.test_updater, ucontext);
    }
}

impl<T> TestComp<T> {
    fn update(&mut self, new_data: T) {
        self.data = new_data;
    }
}

pub struct Test<T: 'static + TestDataInterface> {
    _comp: RcComp<TestComp<T>>,
    callback: CallbackArg<T>,
}

impl<T: 'static + TestDataInterface> Test<T> {
    pub fn set_up(data: T) -> Test<T> {
        let comp = create_component(|_| TestComp { data }, |_, _: ()| {}, |_, _| {});
        let callback = comp.comp().callback_arg(TestComp::update);
        Self {
            _comp: comp,
            callback,
        }
    }

    pub fn update(&self, new_value: T) {
        self.callback.call(new_value);
    }

    fn get_root_node(&self) -> web_sys::Element {
        helper::get_element_by_id("spair_test").unwrap_throw()
    }

    pub fn text_content(&self) -> Option<String> {
        self.get_root_node().text_content()
    }

    pub fn execute_on_root_node<O>(&self, func: impl Fn(&Node) -> O) -> O {
        func(&self.get_root_node())
    }
}
