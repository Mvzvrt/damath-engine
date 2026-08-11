# Damax

[![Build Status](https://img.shields.io/badge/build-manual%20only-orange)](#installation)
[![Version](https://img.shields.io/badge/version-0.1.0-blue)](#installation)
[![License](https://img.shields.io/badge/license-GPL--3.0--only-green)](LICENSE)

<p align="center">
  <img src="gui/public/favicon.svg" alt="Damath Engine logo" width="128" />
</p>

Damax is an open-source Integer Damath engine with a Rust CLI, a WebAssembly-powered core, and a React + Vite browser UI. It lets you play locally, watch the engine search for moves, and reuse the same rules across terminal and browser.

![Gameplay Demo](https://github.com/user-attachments/assets/9cec03ad-7fb7-4725-b40f-e4e57406e86f)

## Prerequisites

- Rust stable toolchain
- `wasm32-unknown-unknown` target for Rust
- `wasm-pack`
- Node.js 18 or newer
- npm

## Installation

```bash
git clone https://github.com/Mvzvrt/damath-engine.git
cd damath-engine
cd nub

rustup target add wasm32-unknown-unknown
cargo install wasm-pack

wasm-pack build --target web

cd gui
npm install
```

## Usage

### Run the GUI

```bash
cd gui
npm run dev
```

Then open the Vite dev server URL shown in the terminal, usually `http://localhost:5173`.

### Run the CLI

```bash
cargo run -p cli
```

The CLI opens an interactive menu where you can start a new game, play against the engine, watch engine-vs-engine play, or run analysis mode.

## Contributing

Contributions are welcome. Keep changes focused, regenerate the WASM package when Rust engine APIs change, and include screenshots or gameplay recordings for visible UI work when possible.

If you touch the engine or WASM bridge, rebuild `nub/pkg` as indicated above and copy the `pkg/` into `gui/src/pkg`. Verify the GUI still starts cleanly with `npm run dev` and `npm run build` locally.

## License

This repository is distributed under the GPL-3.0-only license. See [LICENSE](LICENSE) for the full text.