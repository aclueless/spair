use crate::{
    component::{Checklist, Command as CommandTrait, Comp, Component},
    Command,
};
use std::marker::PhantomData;
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

    pub fn with_fn<C, Cl, Cb>(self, f: Cb) -> Command<C>
    where
        C: Component,
        Cl: 'static + Into<Checklist<C>>,
        Cb: 'static + FnOnce(&mut C, A) -> Cl,
    {
        FcFn {
            future: self.future,
            callback: f,
            phantom: PhantomData,
        }
        .into()
    }
}

struct FutureCallbackFn<F, A, C, Cl, Cb>(Option<FcFn<F, A, C, Cl, Cb>>);

struct FcFn<F, A, C, Cl, Cb> {
    future: F,
    callback: Cb,
    phantom: PhantomData<fn(C, A) -> Cl>,
}

impl<F, A, C, Cl, Cb> CommandTrait<C> for FutureCallbackFn<F, A, C, Cl, Cb>
where
    A: 'static,
    F: 'static + std::future::Future<Output = A>,
    C: Component,
    Cl: 'static + Into<Checklist<C>>,
    Cb: 'static + FnOnce(&mut C, A) -> Cl,
{
    fn execute(&mut self, comp: &Comp<C>, _state: &mut C) {
        let FcFn {
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

impl<F, A, C, Cl, Cb> From<FcFn<F, A, C, Cl, Cb>> for Command<C>
where
    A: 'static,
    F: 'static + std::future::Future<Output = A>,
    C: Component,
    Cl: 'static + Into<Checklist<C>>,
    Cb: 'static + FnOnce(&mut C, A) -> Cl,
{
    fn from(fca: FcFn<F, A, C, Cl, Cb>) -> Self {
        Command(Box::new(FutureCallbackFn(Some(fca))))
    }
}
