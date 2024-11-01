import gleam/list
import gleam/string

pub type Token {
  LeftParen
  RightParen

  EndOfFile
  Unknown(String)
}

pub fn lex(source: String) -> List(Token) {
  do_lex(Lexer(source), []) |> list.reverse
}

fn do_lex(lexer: Lexer, tokens: List(Token)) {
  case next(lexer) {
    #(_, EndOfFile) -> tokens
    #(l, t) -> do_lex(l, [t, ..tokens])
  }
}

type Lexer {
  Lexer(source: String)
}

fn next(lexer: Lexer) -> #(Lexer, Token) {
  case lexer.source {
    "(" <> rest -> #(Lexer(rest), LeftParen)
    ")" <> rest -> #(Lexer(rest), RightParen)

    _ ->
      case string.pop_grapheme(lexer.source) {
        Error(_) -> #(lexer, EndOfFile)
        Ok(#(grapheme, rest)) -> #(Lexer(rest), Unknown(grapheme))
      }
  }
}
