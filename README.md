# Space Race

A retro-like 2D game built in Rust

https://github.com/0xtito/space_race/assets/92818759/38f90a46-61c2-46be-b862-4678a70f1831

## How to run Space Race locally

### Browser

I am using `wasm-server-runner` to run the game in the browser. To learn how to install it, please visit the [official repository](https://github.com/jakobhellermann/wasm-server-runner)

Once you have `wasm-server-runner` installed, you can run the game by running the following command in the root directory of the project:

```bash
cargo run --target wasm32-unknown-unknown
```

### Desktop

To run the game in the desktop, you can run the following command in the root directory of the project:

```bash
cargo run --features  bevy/dynamic_linking
```
