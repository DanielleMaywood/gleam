pub type Foo {
  Foo(a: Int, b: Int)
  Bar(a: Int)
}

pub fn add(lhs: Int, rhs: Int) -> Int {
  lhs + rhs
}

pub fn main() {
  Foo(2, 4).a
}
