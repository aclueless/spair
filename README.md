# spair

A framework for **S**ingle **P**age **A**pplication **i**n **R**ust. Inspired by Simi, Mika and Yew, some parts of source are copied from them.

This project is in its early stage, breaking changes are expected.

## Examples

See in `/examples`

### Run examples

You need

* [Rust](https://www.rust-lang.org/) with `wasm32-unknown-unknown` target.
* `cargo install wasm-pack`
* `cargo install basic-http-server` or use your favorite file-server

In an example folder:

    wasm-pack build --dev --target web
    basic-http-server // or your favorite file-server

Open your browser and visit the correct url. By default, `basic-http-server` serves at `http://127.0.0.1:4000`.

## How Spair works

Spair works similar to [Simi](https://gitlab.com/limira-rs/simi). The big difference is that Simi needs procedural macros to implement the idea, but Spair does not need any macros. That said, a procedural macro/macros can help transform HTML-like code into Spair's Rust code. Such macros were not implemented yet.

The first render of a single page application need to render everything from an empty element. But on the second or subsequence renders, most (or all) of elements are already there. Why re-render, diffing and patching? Spair iterates through the existing elements and modifies them where changes are found.

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
When Spair iterates through an element's child nodes if the expected element was not found, it will be created, and marked as `JustCreated`. Every static attributes and static nodes will only be executed on a `JustCreated` element. Every normal-mode attributes and normal-mode nodes will always be executed.

## Notes

HTML's tags and attributes are implemented as methods in Spair. Names that are conflicted with Rust's keywords are implemented using raw identifers such as `r#type`, `r#for`...