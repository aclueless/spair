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

Sections below provide first looks into Spair.

## Static-mode and update-mode

Spair works by iterating through every elements and attributes/properties in the current DOM, which is empty before the first render, creating new items or modifying existing items, it's the update-mode. But there are elements or attributes that will never change. You can tell Spair to just create them but ignore them when iterating over them later by turn on the static-mode.

| items                    | update-mode      | static-mode            | notes                                                              |
| ------------------------ | ---------------- | ---------------------- | ------------------------------------------------------------------ |
| attributes / properties  | `.attributes()`  | `.static_attributes()` | `static_attributes()` must be called before `attributes()`         |
| elements                 | `.nodes()`       | `.static_nodes()`      | only apply to elements, *not* apply to texts/renderable-items      |
| texts / renderable-items | `.render(value)` | `.r#static(value)`     | not affected by mode introduced by `.nodes()` or `.static_nodes()` |

`.nodes()` and `.static_nodes()` can be switched back and forth as many times as you want.

```rust
element
    .static_attributes()
    .class("class-name") // class="class-name" is added on creation, but ignored on subsequence render
    .attributes()
    .value(&some_value) // will be checked and updated if changed
    .nodes()
    .p(|p| {}) // create and update a <p>
    .render(value) // create and update a text
    .r#static(value) // a create-only text - not affected by `.nodes()`.
    .static_nodes()
    .div(|d| {}) // a create-only <div> (because creating in static-mode)
    .render(value) // an updatable text - not affected by `.static_nodes()`
    .r#static(value) // a create-only text - because of `r#static`, not `static_nodes`

```
* **Important note**: when an element is creating in static mode, all its content will be ignored (not update) in future updates.

```rust
element
    .static_nodes()
    .p(|p| {
        // This closure only execute once on the creation of <p>.
        // In the future update, this closure will be IGNORED,
        // therefore, all child-nodes of <p> will NOT be updated despite
        // being created in update-mode.
        p.nodes()
            .span(|s| {})
            .render(value)
    })
```

## Example
*Look in `/examples` for full examples*

This is the `render` method of `examples/counter`:
```rust
impl spair::Component for State {
    type Routes = ();
    fn render(&self, element: spair::Element<Self>) {
        let comp = element.comp();
        element
            .static_nodes()
            .p(|p| {
                p.static_nodes()
                    .r#static("The initial value is ")
                    .r#static(self.value);
            })
            .r#static(Button("-", comp.handler(State::decrement)))
            .render(self.value)
            .r#static(Button("+", comp.handler(State::increment)));
    }
}
```

## `Render` and `StaticRender` traits

You can split your codes into small pieces by implement [`Render`] or [`StaticRender`] on your data types and pass the values to `.render()` or `.r#static()` respectively.

[`Render`] and [`StaticRender`] are implemented for primitives (`i8`, ..., `u64`, `f32`, `f64`, `bool`, `usize`, `isize`). They are simply converted to strings and rendered as text nodes.

## Access to the component state.

When implementing [`Render`], [`StaticRender`] or [`ListItem`] for your data types, you may want to access the state of your component:

```rust
impl spair::Render<State> for &YourType {
    fn render(self, nodes: spair::Nodes<State>) -> spair::Nodes<State> {
        let state = nodes.state(); // type of `state` is `&State`

        nodes.render(state.value)
    }
}
```

## Reconciliation? - No, you must use [`.match_if()`]

Spair does not do reconciliation, users must do it by themselves. When an expected element is not found, Spair create it, but if Spair found an element at the expected index, Spair just assume it is the expected element. Therefore, when you want to render different elements base on a condition, you must tell Spair to do that via [`.match_if()`].

The following code is extracted from `examples/fetch/src/lib.rs`:
```rust
.nodes()
.match_if(|arm| match self.branch.as_ref() {
    Some(branch) => arm
        // Tell Spair which arm we are on.
        // If in the previous render we were on index=1, but in this
        // render we are on index=0, then Spair clear all nodes before
        // rendering the content
        .render_on_arm_index(0)
        // Render the content of `Some(branch)`
        .render(branch)
        // some code removed
        .done(),
    None => arm
        .render_on_arm_index(1)
        // There is no value: `None`? Then just render a button
        .button(|b| {/* some code removed */})
        .done(),
})
```

**DON'T DO THIS, IT DOES NOT WORK**
```rust
if some_condition {
    nodes().div(|d| {})
} else {
    nodes().p(|p| {})
}
```

## Child components

Spair supports child components, but you do not have to use them if you can avoid them.

Example: `examples/components`

## Notes

HTML's tags and attributes are implemented as methods in Spair. Names that are conflicted with Rust's keywords are implemented using raw identifers such as `r#type`, `r#for`...

## Common errors
Using Spair, you may encounter common mistakes listed in this section. They are really annoying. How these problems can be avoided?
### `static_attributes()`
If you set an attribute in static-mode it will never be updated. It is easy to misplace an update-mode attribute under static-mode.
### arm index in `match_if()`
`match_if()` requires you to specify which arm-index you are on to correctly render/update the content. The number must be unique, otherwise it does not work correctly. And the problem become worse when you refactor your code, it easy forgetting update the indices.

## What's done?

* Non-keyed-list
* Keyed-list (behind `features=["keyed-list"]`)
* Support for `fetch`
    * JSON: `features=["fetch-json"]`
    * RON: `features=["fetch-ron"]`
    * I believe it's ready to add other formats
* Basic support for routing

## What's next?

(But don't expect soon)

- [ ] Using Spair for some apps to stabilize API
- [ ] Documentation
- [ ] Implement `#[derive(spair::Routes)]`
- [x] Add support for child components
- [ ] Some benchmarks
- [ ] Proc macro to convert HTML-like or other short-clear syntax to Spair's Rust code.
- [ ] Using child components is inevitable in complex apps. Is it possible to reduce size added by using multi components?


[Simi]: https://gitlab.com/limira-rs/simi
[Mika]: https://gitlab.com/limira-rs/mika
[Yew]: https://github.com/yewstack/yew
[Rust]: https://www.rust-lang.org/

[`Render`]: https://docs.rs/spair/latest/spair/trait.Render.html
[`StaticRender`]: https://docs.rs/spair/latest/spair/trait.StaticRender.html
[`ListItem`]: https://docs.rs/spair/latest/spair/trait.ListItem.html
[`.match_if()`]: https://docs.rs/spair/latest/spair/dom/nodes/trait.DomBuilder.html#method.match_if
