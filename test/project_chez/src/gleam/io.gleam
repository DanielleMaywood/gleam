import gleam/string

@external(chez, "gleam_stdlib_ffi", "print")
pub fn print(term: String) -> Nil

@external(chez, "gleam_stdlib_ffi", "print_error")
pub fn print_error(term: String) -> Nil

@external(chez, "gleam_stdlib_ffi", "println")
pub fn println(term: String) -> Nil

@external(chez, "gleam_stdlib_ffi", "println_error")
pub fn println_error(term: String) -> Nil

pub fn debug(term: anything) -> anything {
  term |> string.inspect |> println_error
  term
}
