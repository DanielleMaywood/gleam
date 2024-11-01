import gleam/bool_test
import gleam/int
import gleam/list_test

import expect

pub fn main() {
  expect.run_suite([
    can_add_numbers,
    can_construct_and_access_tuple,
    can_bitwise_and,
    can_digits,
    bool_test.and_test,
    bool_test.or_test,
    bool_test.negate_test,
    bool_test.nor_test,
    bool_test.nand_test,
    bool_test.exclusive_or_test,
    bool_test.exclusive_nor_test,
    bool_test.compare_test,
    bool_test.to_int_test,
    bool_test.to_string_test,
    bool_test.guard_test,
    bool_test.lazy_guard_test,
    list_test.length_test,
    list_test.length_test,
  ])
}

fn can_add_numbers() {
  use <- expect.named("can_add_numbers")

  use <- expect.to_be_equal(1 + 1, 2)
  use <- expect.to_be_equal(2 + 2, 4)
  use <- expect.to_be_equal(4 + 3, 7)
  use <- expect.to_be_equal(8 + 4, 12)

  expect.success()
}

fn can_construct_and_access_tuple() {
  use <- expect.named("can_construct_and_access_tuple")

  use <- expect.to_be_equal(#(1, 2, 3).0, 1)
  use <- expect.to_be_equal(#(1, 2, 3).1, 2)
  use <- expect.to_be_equal(#(1, 2, 3).2, 3)

  expect.success()
}

fn can_bitwise_and() {
  use <- expect.named("can_bitwise_and")

  use <- expect.to_be_equal(int.bitwise_and(9, 5), 1)
  use <- expect.to_be_equal(int.bitwise_and(7, 3), 3)
  use <- expect.to_be_equal(int.bitwise_and(7, 11), 3)

  expect.success()
}

fn can_digits() {
  use <- expect.named("can_digits")

  use <- expect.to_be_equal(int.digits(1234, 10), Ok([1, 2, 3, 4]))

  expect.success()
}
