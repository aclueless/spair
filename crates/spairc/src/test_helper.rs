use wasm_bindgen::UnwrapThrowExt;
use web_sys::Node;

use crate::{
    component::{create_component, RcComp},
    helper, CallbackArg, ComponentRoot, Context, Element,
};

pub trait TestDataInterface: Sized {
    type ViewState;
    fn init(&self, root: &Element, context: Context<TestComp<Self>>) -> Self::ViewState;
    fn update(&self, updater: &mut Self::ViewState, context: Context<TestComp<Self>>);
}
pub struct TestComp<T> {
    data: T,
}

pub struct CompUpdater<T: TestDataInterface> {
    _root: Element,
    test_updater: T::ViewState,
}

impl<T: TestDataInterface> crate::Component for TestComp<T> {
    type ViewState = CompUpdater<T>;

    fn init(&self, comp: &crate::Comp<Self>) -> (crate::ComponentRoot, Self::ViewState) {
        let context = crate::Context { comp, state: self };
        let element = Element::with_html("<div id='spair_test'></div>", 0);
        let test_updater = self.data.init(&element, context);
        element.append_to_body();
        (
            ComponentRoot::Body,
            CompUpdater {
                _root: element,
                test_updater,
            },
        )
    }

    fn render(&self, updater: &mut Self::ViewState, comp: &crate::Comp<Self>) {
        let context = crate::Context { comp, state: self };
        self.data.update(&mut updater.test_updater, context);
    }
}

impl<T> TestComp<T> {
    fn update(&mut self, new_data: T) {
        self.data = new_data;
    }
}

pub struct Test<T: TestDataInterface> {
    comp: RcComp<TestComp<T>>,
    callback: CallbackArg<T>,
}

impl<T: 'static + TestDataInterface> Test<T> {
    pub fn set_up(data: T) -> Test<T> {
        let comp = create_component(TestComp { data });
        let callback = comp.comp().callback_arg(TestComp::update);
        Self { comp, callback }
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
