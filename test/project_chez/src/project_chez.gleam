@external(chez, "", "display")
pub fn display(item: anything) -> Nil

pub type Foo {
  Foo(x: Int, y: Int)
}

pub fn fib(n: Int) -> Int {
  do_fib(n, 0, 1)
}

fn do_fib(n: Int, a: Int, b: Int) -> Int {
  case n {
    0 -> a
    1 -> b
    _ -> do_fib(n - 1, b, a + b)
  }
}

pub fn main() {
  #(1, 2).1 + Foo(1, 2).x
}
