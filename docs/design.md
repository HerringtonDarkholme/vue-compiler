# Architecture

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


# Implementation Detail

* Plenty of `debug_assert`s to maintain compiler state invariants.
* The library seeks minimal allocation by using `&str`, `Cow<'_, str>` and `smallvec`.
* A customized `VStr` is used to minimize string manipulation.
* `Fxhash` is preferred over default hasher since hash collision is not a concern.
* The `bitflags` crate is used to represent runtime helper and vnode patch flags.
* Use [heavily optimized](https://github.com/BurntSushi/memchr) routines for string search primitives. ([Perf reference](https://lise-henry.github.io/articles/optimising_strings.html))
* Benchmark with [criterion.rs](https://github.com/bheisler/criterion.rs).
* Test compiler output by [snapshot](https://github.com/mitsuhiko/insta) test.
* Use alternative allocator like [wee_alloc](https://github.com/rustwasm/wee_alloc) or [mi_malloc](https://microsoft.github.io/mimalloc/index.html).
* Use `Box<[T]>` instead of `Vec` to reduce type size.
* Use Arean to minimize allocation.
* A `Future` like stack-allocated transformation `Pass` composition.
* Use `Rc` to manage error handler. Don't optimize wrong code.
* Parallelized conversion with Rayon.

