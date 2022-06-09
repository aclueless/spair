use crate::component::{Checklist, Comp, Component};
use std::rc::Rc;

pub trait FnArg<C: Component, A> {
    fn execute(&self, state: &mut C, a: A) -> Checklist<C>;
}

pub struct Cb<F>(pub F);
impl<C, Cl, F> FnArg<C, ()> for Cb<F>
where
    C: Component,
    Cl: 'static + Into<Checklist<C>>,
    F: Fn(&C) -> Cl,
{
    fn execute(&self, state: &mut C, _: ()) -> Checklist<C> {
        (self.0)(state).into()
    }
}

pub struct CbArg<F>(pub F);
impl<C, Cl, A, F> FnArg<C, A> for CbArg<F>
where
    C: Component,
    Cl: 'static + Into<Checklist<C>>,
    F: Fn(&C) -> Cl,
{
    fn execute(&self, state: &mut C, _a: A) -> Checklist<C> {
        (self.0)(state).into()
    }
}

pub struct CbMut<F>(pub F);
impl<C, Cl, F> FnArg<C, ()> for CbMut<F>
where
    C: Component,
    Cl: 'static + Into<Checklist<C>>,
    F: Fn(&mut C) -> Cl,
{
    fn execute(&self, state: &mut C, _: ()) -> Checklist<C> {
        (self.0)(state).into()
    }
}

pub struct CbArgMut<F>(pub F);
impl<C, Cl, A, F> FnArg<C, A> for CbArgMut<F>
where
    C: Component,
    Cl: 'static + Into<Checklist<C>>,
    F: Fn(&mut C, A) -> Cl,
{
    fn execute(&self, state: &mut C, a: A) -> Checklist<C> {
        (self.0)(state, a).into()
    }
}

pub struct CbDroppedArg<F>(pub F);
impl<C, Cl, A, F> FnArg<C, A> for CbDroppedArg<F>
where
    C: Component,
    Cl: 'static + Into<Checklist<C>>,
    F: Fn(&C) -> Cl,
{
    fn execute(&self, state: &mut C, _a: A) -> Checklist<C> {
        (self.0)(state).into()
    }
}

pub struct CbDroppedArgMut<F>(pub F);
impl<C, Cl, A, F> FnArg<C, A> for CbDroppedArgMut<F>
where
    C: Component,
    Cl: 'static + Into<Checklist<C>>,
    F: Fn(&mut C) -> Cl,
{
    fn execute(&self, state: &mut C, _a: A) -> Checklist<C> {
        (self.0)(state).into()
    }
}

pub struct CallbackFn<C, A>
where
    C: Component,
{
    pub comp: Comp<C>,
    pub callback: Rc<dyn FnArg<C, A>>,
}

impl<C, A> CallbackFn<C, A>
where
    C: Component,
    A: 'static,
{
    pub fn call(&self, a: A) {
        self.comp.execute_callback(a, self);
    }
}

pub trait ExecuteCallback<C: Component, A> {
    fn queue(self, arg: A);
    fn execute(self, state: &mut C, arg: A) -> Checklist<C>;
}

impl<C, A> ExecuteCallback<C, A> for &CallbackFn<C, A>
where
    C: Component,
    A: 'static,
{
    fn queue(self, a: A) {
        let clone = CallbackFn {
            comp: self.comp.clone(),
            callback: self.callback.clone(),
        };
        crate::component::update_component(move || clone.call(a));
    }

    fn execute(self, state: &mut C, a: A) -> Checklist<C> {
        self.callback.execute(state, a)
    }
}

pub struct CallbackFnOnce<C, Cl, F>
where
    C: Component,
    F: FnOnce(&mut C) -> Cl,
{
    pub comp: Comp<C>,
    pub callback: F,
    pub phantom: std::marker::PhantomData<Cl>,
}

impl<C, Cl, F> CallbackFnOnce<C, Cl, F>
where
    C: Component,
    F: 'static + FnOnce(&mut C) -> Cl,
    Cl: 'static + Into<Checklist<C>>,
{
    pub fn call(self) {
        self.comp.clone().execute_callback((), self);
    }
}

impl<C, A, Cl, F> ExecuteCallback<C, A> for CallbackFnOnce<C, Cl, F>
where
    C: Component,
    A: 'static,
    F: 'static + FnOnce(&mut C) -> Cl,
    Cl: 'static + Into<Checklist<C>>,
{
    fn queue(self, _: A) {
        crate::component::update_component(move || self.call());
    }

    fn execute(self, state: &mut C, _: A) -> Checklist<C> {
        (self.callback)(state).into()
    }
}

pub struct CallbackFnOnceArg<C, A, Cl, F>
where
    C: Component,
    F: FnOnce(&mut C, A) -> Cl,
{
    pub comp: Comp<C>,
    pub callback: F,
    pub _phantom: std::marker::PhantomData<fn(A) -> Cl>,
}

impl<C, A, Cl, F> CallbackFnOnceArg<C, A, Cl, F>
where
    C: Component,
    A: 'static,
    F: 'static + FnOnce(&mut C, A) -> Cl,
    Cl: 'static + Into<Checklist<C>>,
{
    pub fn call(self, a: A) {
        self.comp.clone().execute_callback(a, self);
    }
}

impl<C, A, Cl, F> ExecuteCallback<C, A> for CallbackFnOnceArg<C, A, Cl, F>
where
    C: Component,
    A: 'static,
    F: 'static + FnOnce(&mut C, A) -> Cl,
    Cl: 'static + Into<Checklist<C>>,
{
    fn queue(self, a: A) {
        crate::component::update_component(move || self.call(a));
    }

    fn execute(self, state: &mut C, a: A) -> Checklist<C> {
        (self.callback)(state, a).into()
    }
}

pub trait Callback {
    fn call(&self);
}

impl<C> Callback for CallbackFn<C, ()>
where
    C: Component,
{
    fn call(&self) {
        self.call(());
    }
}

pub trait CallbackArg<A> {
    fn call(&self, a: A);
}

impl<C, A> CallbackArg<A> for CallbackFn<C, A>
where
    C: Component,
    A: 'static,
{
    fn call(&self, a: A) {
        self.call(a);
    }
}

pub trait CallbackOnce {
    fn call(self);
}

pub trait CallbackOnceArg<A> {
    fn call(self, a: A);
}

impl<C, Cl, F> CallbackOnce for CallbackFnOnce<C, Cl, F>
where
    C: Component,
    F: 'static + FnOnce(&mut C) -> Cl,
    Cl: 'static + Into<Checklist<C>>,
{
    fn call(self) {
        self.call();
    }
}

impl<C, A, Cl, F> CallbackOnceArg<A> for CallbackFnOnceArg<C, A, Cl, F>
where
    C: Component,
    A: 'static,
    F: 'static + FnOnce(&mut C, A) -> Cl,
    Cl: 'static + Into<Checklist<C>>,
{
    fn call(self, a: A) {
        self.call(a);
    }
}
