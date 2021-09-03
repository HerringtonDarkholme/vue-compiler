# vue template compiler in Rust

https://github.com/vuejs/rfcs/discussions/369#discussioncomment-1192421

> Maybe in the long run we want the whole transform (and even the Vue compiler!) to be implemented in native Go/Rust so performance would no longer be a concern ;)

Future is now!

## Design

The original design in [vue-next](https://github.com/vuejs/vue-next/blob/master/packages/compiler-core/src/ast.ts) mixes
code generation and ast parsing in the same data structure. As we can see, the transform pass will in-place mutate ast nodes,
leaving the node with both code generation node and ssr code generation node.

This is typically a sign of leaky abstraction.
So in the Rust version I decided to take another approach.

The compilation has several phases:
* Scan (output: Token)
* Parse (output: template AST)
* intermediate representation
* transformation/optimization pass
* output generation

## Intended Usage

* Rust library
* CLI binary
* napi based nodejs library
* wasm based npm package

## Implementation Detail

* The library seeks minimal allocation by using `&str`, `Cow<'_, str>` and `smallvec`.
* `Fxhash` is preferred over default hasher since hash collision is not a concern.
* The `bitflags` crate is used to represent runtime helper and vnode patch flags.
* Use [heavily optimized](https://github.com/BurntSushi/memchr) routines for string search primitives.
* Benchmark with [criterion.rs](https://github.com/bheisler/criterion.rs).
* Test compiler output by [snapshot](https://github.com/mitsuhiko/insta) test.

## Reference

* [vue-next](https://github.com/vuejs/vue-next): ご本家様
* [html spec](https://html.spec.whatwg.org/multipage/parsing.html) is the definitive guide for parsing HTML-like files.
* [Vue Template Explorer](https://vue-next-template-explorer.netlify.app/) gives instant results for  code generation and error reporting.
* [Nu html checker](https://validator.w3.org/nu/#textarea) is the official html validator from W3C. This is the canonical error reporter for html parsing, when there is a discrepancy between the framework and the spec.
* [AST explorer](https://astexplorer.net/) can inspect AST nodes interactively.

## Performance Related Reference

* https://lise-henry.github.io/articles/optimising_strings.html

## Roadmap

Todo tasks grouped by scopes.

### [core]
- [x] tokenizer
- [ ] parser
- [ ] IR converter
- [ ] transformer
- [ ] code generator
### [dom]
- [ ] transformer
- [ ] code generator
### [ssr]
- [ ] TODO
### [sfc]
- [ ] TODO
### [test]
- [ ] Add unit test
- [ ] Add insta snapshot
### [bench]
- [x] Add benchmark framework
- [ ] Micro benchmarks for compiler components
- [ ] Integrated benchmarks using repos like [Element-Plus](https://github.com/element-plus/element-plus)
### [infra]
- [x] Add [pre-commit](https://pre-commit.com/) hooks.
- [x] Add Github Actions for various checks.
### [community]
- [ ] TODO. not ready for contribution for now.
