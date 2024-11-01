import gleam/io
import lexer

pub fn main() {
  io.debug(lexer.lex("(abc)"))
}
