import project_chez/math

fn do_math(lhs, rhs, perform) {
  perform(lhs, rhs)
}

pub fn main() {
  do_math(4, 6, math.add)
}
