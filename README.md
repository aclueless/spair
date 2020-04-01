# spair

A framework for *S*ingle *P*age *A*pplication *i*n *R*ust. Inspired by Simi, Mika and Yew, some parts of source are copied from them.

This project is in its early stage, breaking changes are expected.

# Examples

See in `/examples`

## Run examples

In an example folder:

    wasm-pack build --dev --target web
    basic-http-server // installed by `cargo install basic-http-server` or serve with you favorite file server

Open your browser and visit the correct url. By default, `basic-http-server` serves at `http://127.0.0.1:4000`.

# How Spair works

Spair works similar to [Simi](https://gitlab.com/limira-rs/simi). The big difference is that Simi needs procedural macros to implement the idea, but Spair does not need any macros. That said, a procedural macro/macros can help transform HTML-like code into Spair's Rust code. Such macros were not implemented yet.

The first render of a single page application need to render everything from an empty element. But the second or subsequence renders, most (or all) of elements are already there. Why re-render, diffing and patching? Spair iterates through the existing elements and modifies them where changes are found.

