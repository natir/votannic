# votannic 🧬 💻

![tests](https://github.com/natir/votannic/workflows/tests/badge.svg)
![lints](https://github.com/natir/votannic/workflows/lints/badge.svg)
![msrv](https://github.com/natir/votannic/workflows/msrv/badge.svg)
[![website](https://github.com/natir/votannic/workflows/website/badge.svg)](https://natir.github.io/votannic)
[![dev-doc](https://img.shields.io/badge/dev-doc-blue)](https://natir.github.io/votannic/doc/votannic)
[![license](https://img.shields.io/badge/license--purple)](/blob/main/LICENSE)
[![copier](https://img.shields.io/badge/copier-template-yellow)](https://github.com/natir/copier-rust)

A tools to generate all possible outcomes in Warhammer 40k system

## Usage

Add this in your Cargo.toml
```
votannic = { version = "0.1" }
```

### From source

```bash
git clone {repository_uri}
cd
cargo install --path
```

## Feature

- parse newrecuit json
- compute expectation of weapon of one army against body of other
- dice support critical
- variance support

## Todo

- lethal hit
- lethal wound

## Minimum Supported Rust Version

Currently the minimum supported Rust version is 1.92.
