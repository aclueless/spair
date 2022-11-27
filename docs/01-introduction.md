# Introduction

Spair is a Rust frontend framework. Compare to other frameworks in Rust, Spair is both small and fast.

Spair is possibly the weirdest framework in Rust frontend ecosystem. Spair is neither vDOM, nor reactive. But Spair provides features similar to **both** pure-vDOM (without using hooks) and reactive frameworks. The two modes are called *incremental-render* (pure-Spair, or simply Spair for short if not causing confusion) and *queue-render* (Spair-qr). Both modes can be used in the same component.

There are more weirdnesses wait for you if you dive into Spair. But I think it's worth it because pure-Spair is able to reach high performance without having to deal with things like hooks, or signals. If pure-Spair performance is not enough, Spair-qr can help a bit more in performance.

In performance, pure-Spair is significant faster than Yew, React and comparable to [Dominator] and [Sycamore]. Spair-qr is comparable to Leptos and Solid-store. In size, Spair produces smaller `.wasm` file than all of Rust frameworks mentioned above (Yew, Dominator, Sycamore and Leptos).

You can always start to build your components with pure-Spair (it's easy to work with, but still very fast). Then, if there are some performance issues, you can introduce queue-render to the problem parts of your components. It means that just some critical parts of your app works in queue-render mode. The remaining parts of the app still work in incremental mode.

## The `Hello World` app

Spair is designed to make big single page applications (I think so, but I may be wrong). Therefore, it looks a bit overcomplicated with the Hello World exmaple.
```rust
struct State {
    message: String,
}

impl spair::Component for State {
    fn render(&self, element: spair::Element<Self>) {
        // Render the value of `self.message` as a text in `element`
        // on the first render, and update it on subsequence renders
        element.rupdate(&self.message);
    }
}

// The root component is required to implement this,
// child component is required to implement a different trait.
impl spair::Application for State {
    fn init(_: &spair::Comp<Self>) -> Self {
        Self {
            message: "Hello World!".to_string(),
        }
    }
}

fn main() {
    State::mount_to_element_id("root_element_id");
    // or,
    // State::mount_to_body();
}
```

## Weirdnesses cause by incremental render

Incremental render is the default mode in Spair. "Incremental render" means that Spair starts with an empty element in your browser and add elements to it on the first run, then modify those elements on subsequence runs. Therefore, Spair's render method will not return an element, but require an element passed to it. In the Hello World example, Spair will look for and element with `id="root_element_id"` and pass it to the `Component::render` method.

The requirement of a pre-existing element applies to all pieces of render code. For example, you can define a method to render a button:
```rust
fn render_button(button: spair::Element<State>) {
    // Render the button here
}
```
In the code above, `State` (in `spair::Element<State>`) is the type of your component's state. Again, you can see the the `button` is required by `render_button`, so you have to use it like this:
```rust
impl spair::Component for State {
    fn render(&self, element: spair::Element<State>) {
        element.button(render_button);
        //             ^^^^^^^^^^^^^ this is `fn render_button(spair::Element<State>)`
        //      ^^^^^^ `button` is a method that creates a `<button>` as a child of `element`
    }
}
```
Well, you encountered another weirdness in Spair. You have to tell Spair that this is a `<button>` and how to render it (`fn render_button`) in two different pieces of code. Oh, you can actually tell Spair both information in one place, though:
```rust
fn render_buttons(nodes: spair::Nodes<State>) {
    nodes
        // `b` is a `spair::Element<State>`
        .button(|b|{
            // Render the button here
        })
        .button(|b|{
            // Render another button here
        })
}
```
And use it like:
```rust
element
    .rfn(|nodes| render_buttons(nodes));
```
Yeah, you encountered another weirdness, again. But it still is little nice because we can create multi elements in `fn render_buttons()`.

If you has a piece of data that you want to create a render for content you can do like this:
```rust
struct MyData {
    my_value: i32,
}

struct State {
    my_data: MyData,
}

impl spair::Render<State> for &MyData {
    fn render(self, nodes: spair::Nodes<State>) {
        // Render or update the value of `self.my_value` as a text in `nodes` 
        nodes.rupdate(self.my_value);
    }
}

impl spair::Component for State {
    fn render(&self, element: spair::Element<State>) {
        element.rupdate(&self.my_data);
    }
}

```
You can see that both `.my_value` (`i32`) and `.my_data` (`MyData`) were pass to methods with the same name `.rupdate()`. Yes, methods named `.rupdate()` receives any data that implement `spair::Render<State>` trait. The `Render` traits are implement for all primitive types like: `i32`, `u32`, `f32`, `bool`....

## What are benifits of incremental render?

