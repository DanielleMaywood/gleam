@external(chez, "gleam_stdlib_ffi", "inspect")
pub fn inspect(term: anything) -> String

@external(chez, "", "string-grapheme-count")
pub fn length(term: String) -> Int

@external(chez, "gleam_stdlib_ffi", "pop_grapheme")
pub fn pop_grapheme(string: String) -> Result(#(String, String), Nil)
