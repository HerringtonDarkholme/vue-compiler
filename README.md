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
* ~~Possibly a SIMD library for string pattern matching might help performance, like [hyperscan](http://intel.github.io/hyperscan).~~
* Use `codespan` for beautiful diagnostic reporting [codespan](https://github.com/brendanzab/codespan).
