use std::collections::HashMap;
// a dictionary for metavariable instantiation
// const a = 123 matched with const a = $A will produce env: $A => 123
pub type Env = HashMap<String, String>;

pub struct Rule<Matcher> {
    env: Env,
    matcher: Matcher,
}

pub struct And<P1, P2> {
    pattern1: P1,
    pattern2: P2,
}

pub struct Or<P1, P2> {
    pattern1: P1,
    pattern2: P2,
}

pub struct Inside<Outer, Inner> {
    outer: Outer,
    inner: Inner,
}
pub struct NotInside<Outer, Inner> {
    outer: Outer,
    inner: Inner,
}

pub struct Not<P> {
    not: P,
}
