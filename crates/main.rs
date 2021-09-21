struct H<'a>(&'a str);

trait Pass<T, Info> {
    fn enter(&mut self, t: &mut T, i: &mut Info);
}
trait I<'a> {
    fn add_ident(&mut self, name: &'a str);
}
struct Count(i32);
impl<'a, Info> Pass<H<'a>, Info> for Count
where
    Info: I<'a>,
{
    fn enter(&mut self, t: &mut H<'a>, i: &mut Info) {
        self.0 += 1;
        i.add_ident("test");
    }
}

struct Info<'a> {
    v: Vec<&'a str>,
}
impl<'a> I<'a> for Info<'a> {
    fn add_ident(&mut self, name: &'a str) {
        self.v.push(name);
    }
}
fn transform<T>(ps: Vec<Box<dyn Pass<T, Info>>>, t: &mut T) {
    let mut info = Info { v: vec![] };
    for mut p in ps {
        p.enter(t, &mut info)
    }
}

fn main() {
    let mut h = H("hello");
    transform(vec![Box::new(Count(0))], &mut h);
}
