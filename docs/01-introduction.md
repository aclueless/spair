# Introduction

Spair is possibly the weirdest framework in Rust frontend ecosystem. Spair is neither vDOM, nor reactive. But Spair allows you to **work with data** similar to **both** in pure-vDOM (without using hooks) and reactive frameworks. The two modes can be called *incremental-render* (pure-Spair) and *queue-render* (Spair-qr). Both modes can be used in the same component.

Spair is both small and fast.

In performance, pure-Spair is significant faster than Yew and comparable to [Dominator] or [Sycamore]. Spair-qr is comparable to Leptos.

Spair produces smaller `.wasm` file than all of Rust frameworks mentioned above (Yew, Dominator, Sycamore and Leptos).

## pure-Spair

You can always start to build your components with pure-Spair.

