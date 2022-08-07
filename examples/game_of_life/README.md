# Game of Life Example

This implementation is a port from Yew's implementation.

This example boasts a complete implementation of [Conway's Game of Life](https://en.wikipedia.org/wiki/Conway's_Game_of_Life).
You can manually toggle cells by clicking on them or create a random layout by pressing the "Random" button.

## Running

This example is quite resource intensive; it's recommended that you only use it with the `--release` flag:

```bash
trunk serve --release
```

## Concepts

- Uses [`gloo_timer`](https://docs.rs/gloo-timers/latest/gloo_timers/) to automatically step the simulation.
- Logs to the console using the [`weblog`](https://crates.io/crates/weblog) crate.
