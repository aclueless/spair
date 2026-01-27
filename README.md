# Spair

[![Crates.io](https://img.shields.io/crates/v/spair)](https://crates.io/crates/spair)
[![docs.rs](https://img.shields.io/docsrs/spair)](https://docs.rs/spair)
![Build](https://github.com/aclueless/spair/workflows/Rust/badge.svg)

A [small] and [fast] frontend framework for **S**ingle **P**age **A**pplication **i**n **R**ust.

This project is in its *early stage*, things are still missing.

## Why not using Spair?

* Routing is just at the minimum level. You have to implement it manually.
* No support for SSR.
* No event delegation.
* No RSX.
    * Spair uses Rust syntax to describe the HTML fragments, then use Rust attribute-macros to transform it to spair's views or components. This is a kind of abusing Rust, and can be broke by a change in Rust in the future.
* The library is still a WIP.
* No community.

## Why using Spair?

* Both [small] and [fast].
    * This was correct for previous versions. The current version (the main branch) is a complete redesign. No benchmarks for the new design yet.
* No vDOM.
    * Spair use procedure macro to find the exact nodes that need to be updated at compile time. Then in updating, it just updates the nodes directly. Items in lists still need to be located by their keys before doing updates.
* Plan to addl: reactive-like update.
    * Queue render (reactive-like, but not using any kind of signals), just queue relevant pieces of code to render change on data change. (Old versions had this, but not available in new design in main branch. It is planned to be added back)
* Rust-like syntax
    * Auto format by rustfmt.
    * (But can be broke by a change in Rust in the future)
* Routing (but just basic support).
* Missing things here and there...
    * Errr, this is definitely a _why not_, obviously. I just put this here to remind potential users not to surprise about missing things :D.

## Run examples

Prerequisites:

* [Rust] with `wasm32-unknown-unknown` target.
* [Trunk] 

In an example folder:

    trunk serve

or, if it's slow, use: (especialy `examples/boids` or `examples/game_of_life`)

    trunk serve --release

Open your browser at http://localhost:8080

## Documentation

Not yet. `/examples/*` is the best place to start now.

Sections below provide first looks into Spair.

## First look

### Views
The following code create a new view name `UpdownButton`. It will never be updated.
```rust
#[create_view]
impl UpdownButton {
    fn create(handler: CallbackArg<MouseEvent>, text: &str) {}
    fn update() {}
    fn view() {
        button(on_click = handler, text(text))
    }
}
```
### Components
The following code `impl` Spair's `Component` trait for `AppState`. Values get from `ucontext` will be updated.
struct AppState {
    value: i32,
}
```rust
#[impl_component]
impl AppState {
    fn create(ccontext: &Context<Self>) {}
    fn update(ucontext: &Context<Self>) {}
    fn view() {
        div(
            replace_at_element_id = "root",
            UpdownButton(ccontext.comp.callback_arg(|state, _| state.value -= 1), "-"),
            ucontext.state.value,
            UpdownButton(ccontext.comp.callback_arg(|state, _| state.value += 1), "+"),
        )
    }
}
// Start the app
fn main() {
    spair::start_app(|_| AppState { value: 42 });
}
```
## Notes

### Names conflict with Rust keywords
HTML's tags and attributes are implemented as methods in Spair. Names that
are conflicted with Rust's keywords are implemented using raw identifers
such as `r#type`, `r#for`...

[Rust]: https://www.rust-lang.org/
[Trunk]: https://trunkrs.dev/

[small]: https://github.com/aclueless/rust-frontend-framework-comparision/tree/main/todomvc
[fast]: https://github.com/krausest/js-framework-benchmark
