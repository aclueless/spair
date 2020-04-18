# spair

![Crates.io](https://img.shields.io/crates/v/spair)
![Build](https://github.com/aclueless/spair/workflows/Rust/badge.svg)

A framework for **S**ingle **P**age **A**pplication **i**n **R**ust. Inspired by [Simi], [Mika] and [Yew], some parts of source are copied from them.

This project is in its early stage, breaking changes are expected.

## Run examples

Prerequisites:

* [Rust] with `wasm32-unknown-unknown` target.
* `cargo install wasm-pack`
* `cargo install basic-http-server` or use your favorite file-server

To build and serve (in an example folder):

    wasm-pack build --dev --target web
    basic-http-server // or your favorite file-server

Open your browser and visit the correct url. By default, `basic-http-server` serves at http://localhost:4000.

## Documentation

Not yet. `/examples/*` is the best place to start now.

## How Spair works

Spair works similar to [Simi]. The big difference is that Simi needs procedural macros to implement the idea, but Spair does not need any macros. That said, a procedural macro/macros can help transform HTML-like code into Spair's Rust code. Such macros were not implemented yet.

The first render of a single page application need to render everything from an empty DOM. But on the second or subsequence renders, most (or all) of elements are already there. Why re-render, diffing and patching? Spair modifies the current DOM, if the expected element is not found, Spair create it, otherwise, just modify where changes occurs.

When implementing `spair::Component` on your application's State, the `spair::Component::render` method receive a `context` which contains a `comp: spair::Comp<C>` and an `element: spair::Element<C>`.

`comp` is used to construct event handlers (for element events) or callbacks. `element` is the root element of your component, on which you can set `static_attributes` and/or `attributes` (in that order). And finally you can add `static_nodes` or `nodes`.

```rust
impl spair::Component for State {
    type Routes = ();
    fn render(&self, c: spair::Context<Self>) {
        let (comp, element) = c.into_parts();
        element
            // This return an object that allow you setting attributes in static mode
            .static_attributes()
            // This class-name only set when element is marked as `JustCreated`, otherwise Spair ignores it
            .class("some-class")
            // This return an object that allow you setting attributes in normal mode
            .attributes()
            // This class-name will be checked and set if the condition changes, every time `fn render` executes.
            .class_if("option-class", self.some_bool_value)
            // This return an object that allow you adding children in static mode
            .static_nodes()
            // The next element (and its descendants) will be rendered only when it does not exist yet,
            // If Spair finds out that it is already there, then Spair just iterates over it.
            .p(|p| {
                // `p` is an element which is the same type as the `element` in spair::Context
                p
                    // Don't care about attributes of this element (<p>)
                    // Just add content for it
                    .nodes()
                    // Render as text if the value is primitive type
                    .render(&self.some_string_value)
                    .render(self.some_copyable_value)
                    // Also render as text if the value is primitive type
                    .r#static(self.some_value_that_renders_as_static);
            })
            // This return an object that allow you adding children in normal mode
            .nodes()
            // The next element (and its descendants) will be processed every time that `fn render` executes
            .p(|p| {
                // sets attributes and/or adds children for `p`
            })
            // Add other nodes in static mode
            .static_nodes()
            .p(|p| {});
    }
}
```

## Notes

HTML's tags and attributes are implemented as methods in Spair. Names that are conflicted with Rust's keywords are implemented using raw identifers such as `r#type`, `r#for`...

## What's done?

* Minimal features of the framework
* Some common events on html elements
* Some attributes (enough for `examples/totomvc`)
* Non-keyed-list
* Keyed-list (behind `features=["keyed-list]`)
* Basic support for `fetch`
* Basic support for routing

## What's next?

(But don't expect soon)

- [ ] Using Spair for some apps to stabilize API
- [ ] Documentation
- [ ] Implement `#[derive(spair::Routes)]`
- [ ] Add support for child components
- [ ] Some benchmarks
- [ ] Proc macro to convert HTML-like or other short-clear syntax to Spair's Rust code.


[Simi]: https://gitlab.com/limira-rs/simi
[Mika]: https://gitlab.com/limira-rs/mika
[Yew]: https://github.com/yewstack/yew
[Rust]: https://www.rust-lang.org/