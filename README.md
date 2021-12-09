# Vue Compiler in Rust

<p align="center">
<a href="https://rustwasm.github.io/wasm-pack/"><img src="https://raw.githubusercontent.com/HerringtonDarkholme/vue-compiler/main/playground/src/assets/wasm-ferris.png" alt="WebAssembly ferris" width="317"></a>
 <a href="https://github.com/vuejs/vue-next"><img src="https://raw.githubusercontent.com/HerringtonDarkholme/vue-compiler/main/playground/src/assets/logo.png" alt="Vue" width="200"></a>
</p>

<p align="center"><a href="https://herringtondarkholme.github.io/vue-compiler/">Try it out in the wasm playground!</a></p>

![CI](https://github.com/HerringtonDarkholme/vue-compiler/actions/workflows/ci.yml/badge.svg)
![Playground](https://github.com/HerringtonDarkholme/vue-compiler/actions/workflows/gh-pages.yml/badge.svg)
[![codecov](https://codecov.io/gh/HerringtonDarkholme/vue-compiler/branch/main/graph/badge.svg?token=A37GRLYA5R)](https://codecov.io/gh/HerringtonDarkholme/vue-compiler)


Evan [says](https://github.com/vuejs/rfcs/discussions/369#discussioncomment-1192421):

> Maybe in the long run we want the whole transform (and even the Vue compiler!) to be implemented in native Go/Rust so performance would no longer be a concern ;)

Future is now!

## Project Docs
* [Architecture Design](https://github.com/HerringtonDarkholme/vue-compiler/blob/main/docs/design.md)
* [Project Roadmap](https://github.com/HerringtonDarkholme/vue-compiler/blob/main/docs/roadmap.md)
* [Contributing Guide](https://github.com/HerringtonDarkholme/vue-compiler/blob/main/.github/CONTRIBUTING.md)


## Intended Usage

* Rust library
* CLI binary
* napi based nodejs library
* wasm based npm package: a fallback if napi fails to work and a toy for browser.
* No Browser build
No support since most features in full build are additional except for browser based expression checking or HTML escaping. Browser build removed them for size. But template compiler in browser is already for toy project. For browser specific summary see [this google sheet](https://docs.google.com/spreadsheets/d/1Uofb9qW9-gxdSh8lbC-CE0kWkhpAAtTFDZlw9UW0HrE/edit?usp=sharing).

## Reference

* [vue-next](https://github.com/vuejs/vue-next): ご本家様
* [html spec](https://html.spec.whatwg.org/multipage/parsing.html) is the definitive guide for parsing HTML-like files.
* [Vue Template Explorer](https://vue-next-template-explorer.netlify.app/) gives instant results for  code generation and error reporting.
* [Nu html checker](https://validator.w3.org/nu/#textarea) is the official html validator from W3C. This is the canonical error reporter for html parsing, when there is a discrepancy between the framework and the spec.
* [AST explorer](https://astexplorer.net/) can inspect AST nodes interactively.

## Benchmark
[benchmark](https://herringtondarkholme.github.io/vue-compiler/dev/bench/) you could inspect the benchmark result here.
