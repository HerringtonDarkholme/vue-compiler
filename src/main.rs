trait NodeTransformer {
    fn pre_trans(&self, n: &mut i32);
    fn post_trans(&self, n: &mut i32);
}
struct T;
impl NodeTransformer for T {
    fn pre_trans(&self, n: &mut i32) {
        *n += 1;
        println!("pre trans: {}", n);
    }
    fn post_trans(&self, n: &mut i32) {
        *n -= 1;
        println!("post trans: {}", n);
    }
}
fn main() {
    let mut node = 123;
    let v = vec![T, T];
    for t in v.iter() {
        t.pre_trans(&mut node);
    }
    for t in v.iter().rev() {
        t.post_trans(&mut node);
    }
}
