## Roadmap

Todo tasks grouped by scopes.

### [util]
- [x] VStr
    - [ ] string intern
    - [ ] camel/pascal cache
    - [ ] str ops
### [core]
- [x] scanner
    - [x] UTF8 support
- [x] parser
- [x] IR converter
    - [x] v-if
    - [x] v-for
    - [x] v-slot
    - [x] v-model
    - [x] slot outlet
    - [x] element
    - [x] build props
- [x] transformer
    - [x] ~~SWC~~ RSLint integration
    - [x] Rewrite MergePass struct
- [x] code generator
    - [x] module preamble
- [x] wrap error handler in Rc
- [x] compile option
- [ ] Arena allocation
- [x] ~~Parallelization~~
### [dom]
- [ ] IR converter
    - [x] v-on
    - [x] v-once
    - [x] v-memo
- [ ] transformer
- [ ] code generator
### [ssr]
- [ ] TODO
### [sfc]
- [ ] script
- [ ] template
    - [ ] asset url

- [ ] style
    - [ ] scoped style
    - [ ] v-bind css var
    - [ ] css modules
### [test]
- [ ] scanner test
- [ ] parser test
    - [x] dir parser test
- [x] Add insta snapshot
- [x] Move snapshot outside of src
### [bench]
- [x] Add benchmark framework
- [ ] Micro benchmarks for scanner
- [ ] Micro benchmarks for parser
- [ ] Micro benchmarks for converter
- [ ] Micro benchmarks for transformer
- [ ] Micro benchmarks for codegen
- [ ] Integrated benchmarks using repos like [Element-Plus](https://github.com/element-plus/element-plus)
### [infra]
- [x] Add [pre-commit](https://pre-commit.com/) hooks.
- [x] Add Github Actions for various checks.
- [x] Change single lib to cargo workspaces.
### [community]
- [ ] TODO. not ready for contribution for now.
