// struct S;
// impl spair::Component for S {
//     type Routes = Route;
//     fn register_routing_callback(router: &mut Self::Routes::Router) {};
// }

thread_local! {
    static ROUTER: Box<dyn Router> = Box::new(());
}

pub trait Router {
    fn routing(&self, location: web_sys::Location);
}

pub trait Routes {
    type Router: Router;
    /// Just help creating a `ghost router` for application that has `type Routes = ();`
    /// You never need to override this method. But you should override `Application::init_router`
    /// to provide your actual Router instance, if not your app will fail immediately.
    /// This method was put here to
    fn unit_router() -> Self::Router {
        unreachable!(
            "You must implement method `Application::init_router` and provide the actual router instance"
        )
    }
    fn url(&self) -> String;
}

impl Router for () {
    fn routing(&self, _: web_sys::Location) {}
}

impl Routes for () {
    type Router = ();
    fn unit_router() -> Self {}
    fn url(&self) -> String {
        String::new()
    }
}
