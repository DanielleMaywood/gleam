import gleam/int
import gleam/order

pub fn all(in items: List(a), satisfying predicate: fn(a) -> Bool) -> Bool {
  case items {
    [] -> True
    [first, ..rest] ->
      case predicate(first) {
        True -> all(rest, predicate)
        False -> False
      }
  }
}

pub fn any(in items: List(a), satisfying predicate: fn(a) -> Bool) -> Bool {
  case items {
    [] -> False
    [first, ..rest] ->
      case predicate(first) {
        True -> True
        False -> any(rest, predicate)
      }
  }
}

pub fn append(first: List(a), second: List(a)) -> List(a) {
  do_append(reverse(first), second)
}

fn do_append(first: List(a), second: List(a)) -> List(a) {
  case first {
    [] -> second
    [item, ..rest] -> do_append(rest, [item, ..second])
  }
}

pub fn count(list: List(a), where predicate: fn(a) -> Bool) -> Int {
  fold(list, 0, fn(acc, value) {
    case predicate(value) {
      True -> acc + 1
      False -> acc
    }
  })
}

pub fn fold(
  over list: List(a),
  from initial: acc,
  with fun: fn(acc, a) -> acc,
) -> acc {
  case list {
    [] -> initial
    [x, ..rest] -> fold(rest, fun(initial, x), fun)
  }
}

@external(chez, "", "length")
pub fn length(of list: List(a)) -> Int

pub fn map(items: List(a), with fun: fn(a) -> b) -> List(b) {
  do_map(items, fun, [])
}

fn do_map(items: List(a), fun: fn(a) -> b, acc: List(b)) -> List(b) {
  case items {
    [] -> reverse(acc)
    [x, ..xs] -> do_map(xs, fun, [fun(x), ..acc])
  }
}

pub fn range(from start: Int, to stop: Int) -> List(Int) {
  tail_recursive_range(start, stop, [])
}

fn tail_recursive_range(start: Int, stop: Int, acc: List(Int)) -> List(Int) {
  case int.compare(start, stop) {
    order.Eq -> [stop, ..acc]
    order.Gt -> tail_recursive_range(start, stop + 1, [stop, ..acc])
    order.Lt -> tail_recursive_range(start, stop - 1, [stop, ..acc])
  }
}

pub fn reverse(xs: List(a)) -> List(a) {
  do_reverse(xs, [])
}

fn do_reverse(remaining: List(a), accumulator: List(a)) -> List(a) {
  case remaining {
    [] -> accumulator
    [item, ..rest] -> do_reverse(rest, [item, ..accumulator])
  }
}
