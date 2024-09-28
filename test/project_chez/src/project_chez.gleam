import gleam/bool
import project_chez/io

pub fn main() {
  io.println(bool.to_string(bool.nor(False, False)))
  io.println(bool.to_string(bool.nor(False, True)))
  io.println(bool.to_string(bool.nor(True, False)))
  io.println(bool.to_string(bool.nor(True, True)))

  Nil
}
