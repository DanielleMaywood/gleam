import gleam/io
import gleam/list
import gleam/option.{type Option, None, Some}
import gleam/string

pub type TestInfo {
  TestInfo(name: String, failure: Option(String))
}

pub type Test =
  fn() -> TestInfo

pub fn run_suite(tests: List(Test)) {
  tests
  |> list.map(fn(run) {
    let info = run()

    case info.failure {
      Some(r) -> io.println("✕ test `" <> info.name <> "` failed: " <> r)
      None -> io.println("✓ test `" <> info.name <> "` passed")
    }
  })
}

pub fn named(name: String, run: fn() -> Option(String)) -> TestInfo {
  TestInfo(name:, failure: run())
}

pub fn success() -> Option(String) {
  None
}

pub fn to_be_equal(
  lhs: a,
  rhs: a,
  next: fn() -> Option(String),
) -> Option(String) {
  case lhs == rhs {
    True -> next()
    False ->
      Some(
        "not equal: `"
        <> string.inspect(lhs)
        <> "` and `"
        <> string.inspect(rhs)
        <> "`",
      )
  }
}

pub fn to_be_true(lhs: Bool, next: fn() -> Option(String)) -> Option(String) {
  case lhs == True {
    True -> next()
    False -> Some("not true: `" <> string.inspect(lhs) <> "`")
  }
}

pub fn to_be_false(lhs: Bool, next: fn() -> Option(String)) -> Option(String) {
  case lhs == False {
    True -> next()
    False -> Some("not false: `" <> string.inspect(lhs) <> "`")
  }
}
