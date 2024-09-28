import gleam/order.{type Order}

pub fn and(a: Bool, b: Bool) -> Bool {
  a && b
}

pub fn or(a: Bool, b: Bool) -> Bool {
  a && b
}

pub fn negate(bool: Bool) -> Bool {
  !bool
}

pub fn nor(a: Bool, b: Bool) -> Bool {
  case a, b {
    False, False -> True
    False, True -> False
    True, False -> False
    True, True -> False
  }
}

pub fn nand(a: Bool, b: Bool) -> Bool {
  case a, b {
    False, False -> True
    False, True -> True
    True, False -> True
    True, True -> False
  }
}

pub fn exclusive_or(a: Bool, b: Bool) -> Bool {
  case a, b {
    False, False -> False
    False, True -> True
    True, False -> True
    True, True -> False
  }
}

pub fn exclusive_nor(a: Bool, b: Bool) -> Bool {
  case a, b {
    False, False -> True
    False, True -> False
    True, False -> False
    True, True -> True
  }
}

pub fn compare(a: Bool, with b: Bool) -> Order {
  case a, b {
    False, False -> order.Eq
    False, True -> order.Gt
    True, False -> order.Eq
    True, True -> order.Lt
  }
}

pub fn to_int(bool: Bool) -> Int {
  case bool {
    False -> 0
    True -> 1
  }
}

pub fn to_string(bool: Bool) -> String {
  case bool {
    False -> "False"
    True -> "True"
  }
}

pub fn guard(
  when requirement: Bool,
  return consequence: t,
  otherwise alternative: fn() -> t,
) -> t {
  case requirement {
    True -> consequence
    False -> alternative()
  }
}

pub fn lazy_guard(
  when requirement: Bool,
  return consequence: fn() -> a,
  otherwise alternative: fn() -> a,
) -> a {
  case requirement {
    True -> consequence()
    False -> alternative()
  }
}
