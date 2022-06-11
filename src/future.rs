use crate::component::{Checklist, Comp, Component};
use wasm_bindgen::UnwrapThrowExt;

pub struct Future<F> {
    future: F,
}

impl<F, A> Future<F>
where
    A: 'static,
    F: 'static + std::future::Future<Output = A>,
{
    pub fn new(future: F) -> Self {
        Self { future }
    }

    pub fn callback<C, Cl, Cb>(self, f: Cb) -> crate::Command<C>
    where
        C: Component,
        Cl: 'static + Into<Checklist<C>>,
        Cb: 'static + FnOnce(&mut C, A) -> Cl,
    {
        Fc {
            future: self.future,
            callback: f,
            phantom: std::marker::PhantomData,
        }
        .into()
    }
}

struct FutureCallback<F, A, C, Cl, Cb>(Option<Fc<F, A, C, Cl, Cb>>);

struct Fc<F, A, C, Cl, Cb> {
    future: F,
    callback: Cb,
    phantom: std::marker::PhantomData<fn(C, A) -> Cl>,
}

impl<F, A, C, Cl, Cb> crate::component::Command<C> for FutureCallback<F, A, C, Cl, Cb>
where
    A: 'static,
    F: 'static + std::future::Future<Output = A>,
    C: Component,
    Cl: 'static + Into<Checklist<C>>,
    Cb: 'static + FnOnce(&mut C, A) -> Cl,
{
    fn execute(&mut self, comp: &Comp<C>, _state: &mut C) {
        let Fc {
            phantom: _,
            future,
            callback,
        } = self
            .0
            .take()
            .expect_throw("Internal error: Why FutureCallback is executed twice?");

        let callback = comp.callback_once_arg_mut(callback);
        let f = async move {
            let rs = future.await;
            callback.call(rs); // .queue(rs) does not work in future, there is no way to execute the update queue now.
        };
        wasm_bindgen_futures::spawn_local(f);
    }
}

impl<F, A, C, Cl, Cb> From<Fc<F, A, C, Cl, Cb>> for crate::Command<C>
where
    A: 'static,
    F: 'static + std::future::Future<Output = A>,
    C: crate::component::Component,
    Cl: 'static + Into<Checklist<C>>,
    Cb: 'static + FnOnce(&mut C, A) -> Cl,
{
    fn from(fca: Fc<F, A, C, Cl, Cb>) -> Self {
        crate::Command(Box::new(FutureCallback(Some(fca))))
    }
}
