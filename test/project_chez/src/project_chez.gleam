@external(chez, "", "display")
pub fn display(item: anything) -> Nil

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
  display(fib(100))
}
