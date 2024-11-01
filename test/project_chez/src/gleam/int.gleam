import gleam/order.{type Order}

pub fn absolute_value(x: Int) -> Int {
  case x >= 0 {
    True -> x
    False -> x * -1
  }
}

pub fn add(a: Int, b: Int) -> Int {
  a + b
}

@external(chez, "", "bitwise-and")
pub fn bitwise_and(x: Int, y: Int) -> Int

@external(chez, "", "bitwise-xor")
pub fn bitwise_exclusive_or(x: Int, y: Int) -> Int

@external(chez, "", "bitwise-not")
pub fn bitwise_not(x: Int) -> Int

@external(chez, "", "bitwise-ior")
pub fn bitwise_or(x: Int, y: Int) -> Int

@external(chez, "", "bitwise-arithmetic-shift-left")
pub fn bitwise_shift_left(x: Int, y: Int) -> Int

@external(chez, "", "bitwise-arithmetic-shift-right")
pub fn bitwise_shift_right(x: Int, y: Int) -> Int

pub fn clamp(x: Int, min min_bound: Int, max max_bound: Int) -> Int {
  x
  |> min(max_bound)
  |> max(min_bound)
}

pub fn compare(a: Int, with b: Int) -> Order {
  case a == b {
    True -> order.Eq
    False ->
      case a < b {
        True -> order.Lt
        False -> order.Gt
      }
  }
}

pub fn digits(x: Int, base: Int) -> Result(List(Int), Nil) {
  case base < 2 {
    True -> Error(Nil)
    False -> Ok(do_digits(x, base, []))
  }
}

fn do_digits(x: Int, base: Int, acc: List(Int)) -> List(Int) {
  case absolute_value(x) < base {
    True -> [x, ..acc]
    False -> do_digits(x / base, base, [x % base, ..acc])
  }
}

pub fn divide(dividend: Int, by divisor: Int) -> Result(Int, Nil) {
  case divisor {
    0 -> Error(Nil)
    divisor -> Ok(dividend / divisor)
  }
}

pub fn is_odd(x: Int) -> Bool {
  x % 2 != 0
}

pub fn max(a: Int, b: Int) -> Int {
  case a > b {
    True -> a
    False -> b
  }
}

pub fn min(a: Int, b: Int) -> Int {
  case a < b {
    True -> a
    False -> b
  }
}
