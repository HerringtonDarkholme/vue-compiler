# vue template compiler in Rust

https://github.com/vuejs/rfcs/discussions/369#discussioncomment-1192421

> Maybe in the long run we want the whole transform (and even the Vue compiler!) to be implemented in native Go/Rust so performance would no longer be a concern ;)

Future is now!

## Architecture

The original design in [vue-next](https://github.com/vuejs/vue-next/blob/master/packages/compiler-core/src/ast.ts) mixes
code generation and ast parsing in the same data structure. As we can see, the transform pass will in-place mutate ast nodes,
leaving the node with both code generation node and ssr code generation node.

This is typically a sign of leaky abstraction.

So in the Rust version I decided to take another approach.

The design targets at three different levels of developers in Vue ecosystem:

* Lib/App creator: every Vue developers who write component library or application code.
* Platform developer: Devs who write compiler implementation for DOM/SSR/Native platform.
* Framework author: Vue core lib author a.k.a Evan.

The core framework targets multiple platforms and can be extended to support more.
Core framework components span all platforms and are hardwired to the core lib runtime.

Platforms are usually DOM or SSR environment. Hosts are browser and node, respectively.
Developing a platform needs to write code for both vue-compiler and vue-runtime.
Optionally platform developer can write code in host, e.g. in hybrid app or mini-program.

And finally lib or app creators can write component library, business code or
application components targeted to certain platforms.

The compilation has several phases:
* Scan (output: Tokens): Hardwired in the compiler at framework level.
* Parse (output: template AST): Hardwired in the compiler at framework level.
* Convert (output: intermediate representation): Customizable for platform developers with sensible default.
* Transform (input/output: customizable IR): Customizable with default by using generic/traits.
* Code Generate (customizable output: e.g. JS/TS): Customizable with default.

## Other Design different from the original compiler
* Directive parsing is implemented manually instead of by regexp.
* [`nodeTransforms`](https://github.com/vuejs/vue-next/blob/642710ededf51f1e57286496ab0a64a4d27be800/packages/compiler-core/src/options.ts#L174) is not supported. It's too hard for app creator to use and maintain IR invariants. Platform devs can still customize by implementing converter/transformer.
* [`directiveTransforms`](https://github.com/vuejs/vue-next/blob/642710ededf51f1e57286496ab0a64a4d27be800/packages/compiler-core/src/options.ts#L179) now can returns not only `Props` but also `SimpleExpression`. The extra flexibility makes a more cohesive v-bind/v-on conversion: the logic for processing the directives now resides in one single file without regard to the presence of an argument.
* Runtime helper collection `context.helper/helperString` is moved out from convert and tracked in transform phase, avoiding several methods and reducing HashMap to a bitflag.

## Intended Usage

* Rust library
* CLI binary
* napi based nodejs library
* wasm based npm package: a fallback if napi fails to work and a toy for browser.
* No Browser build
No support since most features in full build are additional except for browser based expression checking or HTML escaping. Browser build removed them for size. But template compiler in browser is already for toy project. For browser specific summary see [this google sheet](https://docs.google.com/spreadsheets/d/1Uofb9qW9-gxdSh8lbC-CE0kWkhpAAtTFDZlw9UW0HrE/edit?usp=sharing).

## Implementation Detail

* Plenty of `debug_assert`s to maintain compiler state invariants.
* The library seeks minimal allocation by using `&str`, `Cow<'_, str>` and `smallvec`.
* A customized `VStr` is used to minimize string manipulation.
* `Fxhash` is preferred over default hasher since hash collision is not a concern.
* The `bitflags` crate is used to represent runtime helper and vnode patch flags.
* Use [heavily optimized](https://github.com/BurntSushi/memchr) routines for string search primitives. ([Perf reference](https://lise-henry.github.io/articles/optimising_strings.html))
* Benchmark with [criterion.rs](https://github.com/bheisler/criterion.rs).
* Test compiler output by [snapshot](https://github.com/mitsuhiko/insta) test.
* Use alternative allocator like [wee_alloc](https://github.com/rustwasm/wee_alloc).

## Reference

* [vue-next](https://github.com/vuejs/vue-next): ご本家様
* [html spec](https://html.spec.whatwg.org/multipage/parsing.html) is the definitive guide for parsing HTML-like files.
* [Vue Template Explorer](https://vue-next-template-explorer.netlify.app/) gives instant results for  code generation and error reporting.
* [Nu html checker](https://validator.w3.org/nu/#textarea) is the official html validator from W3C. This is the canonical error reporter for html parsing, when there is a discrepancy between the framework and the spec.
* [AST explorer](https://astexplorer.net/) can inspect AST nodes interactively.

## Roadmap

Todo tasks grouped by scopes.

### [util]
- [x] VStr
    - [ ] string intern
    - [ ] camel/pascal cache
    - [ ] str ops
### [core]
- [x] tokenizer
    - [ ] UTF8 support
- [x] parser
- [ ] IR converter
    - [x] v-if
    - [x] v-for
    - [x] v-slot
    - [ ] v-model
    - [x] slot outlet
    - [x] element
    - [x] build props
- [ ] transformer
- [ ] code generator
### [dom]
- [ ] IR converter
    - [ ] v-on
    - [ ] v-once
    - [ ] v-memo
- [ ] transformer
- [ ] code generator
### [ssr]
- [ ] TODO
### [sfc]
- [ ] TODO
### [test]
- [ ] tokenizer test
- [ ] parser test
    - [x] dir parser test
- [x] Add insta snapshot
### [bench]
- [x] Add benchmark framework
- [ ] Micro benchmarks for compiler components
- [ ] Integrated benchmarks using repos like [Element-Plus](https://github.com/element-plus/element-plus)
### [infra]
- [x] Add [pre-commit](https://pre-commit.com/) hooks.
- [x] Add Github Actions for various checks.
- [x] Change single lib to cargo workspaces.
### [community]
- [ ] TODO. not ready for contribution for now.
