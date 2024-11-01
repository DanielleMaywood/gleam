import expect
import gleam/bool
import gleam/order

pub fn and_test() {
  use <- expect.named("bool.and_test")

  use <- expect.to_be_true(bool.and(True, True))
  use <- expect.to_be_false(bool.and(True, False))
  use <- expect.to_be_false(bool.and(False, True))
  use <- expect.to_be_false(bool.and(False, False))

  expect.success()
}

pub fn or_test() {
  use <- expect.named("bool.or_test")

  use <- expect.to_be_true(bool.or(True, True))
  use <- expect.to_be_true(bool.or(False, True))
  use <- expect.to_be_true(bool.or(True, False))
  use <- expect.to_be_false(bool.or(False, False))

  expect.success()
}

pub fn negate_test() {
  use <- expect.named("bool.negate_test")

  use <- expect.to_be_false(bool.negate(True))
  use <- expect.to_be_true(bool.negate(False))

  expect.success()
}

pub fn nor_test() {
  use <- expect.named("bool.nor_test")

  use <- expect.to_be_false(bool.nor(True, True))
  use <- expect.to_be_false(bool.nor(True, False))
  use <- expect.to_be_false(bool.nor(False, True))
  use <- expect.to_be_true(bool.nor(False, False))

  expect.success()
}

pub fn nand_test() {
  use <- expect.named("bool.nand_test")

  use <- expect.to_be_false(bool.nand(True, True))
  use <- expect.to_be_true(bool.nand(True, False))
  use <- expect.to_be_true(bool.nand(False, True))
  use <- expect.to_be_true(bool.nand(False, False))

  expect.success()
}

pub fn exclusive_or_test() {
  use <- expect.named("bool.exclusive_or_test")

  use <- expect.to_be_false(bool.exclusive_or(True, True))
  use <- expect.to_be_true(bool.exclusive_or(True, False))
  use <- expect.to_be_true(bool.exclusive_or(False, True))
  use <- expect.to_be_false(bool.exclusive_or(False, False))

  expect.success()
}

pub fn exclusive_nor_test() {
  use <- expect.named("bool.exclusive_nor_test")

  use <- expect.to_be_true(bool.exclusive_nor(True, True))
  use <- expect.to_be_false(bool.exclusive_nor(True, False))
  use <- expect.to_be_false(bool.exclusive_nor(False, True))
  use <- expect.to_be_true(bool.exclusive_nor(False, False))

  expect.success()
}

pub fn compare_test() {
  use <- expect.named("bool.compare_test")

  use <- expect.to_be_equal(bool.compare(True, True), order.Eq)
  use <- expect.to_be_equal(bool.compare(True, False), order.Gt)
  use <- expect.to_be_equal(bool.compare(False, True), order.Lt)
  use <- expect.to_be_equal(bool.compare(False, False), order.Eq)

  expect.success()
}

pub fn to_int_test() {
  use <- expect.named("bool.to_int_test")

  use <- expect.to_be_equal(bool.to_int(True), 1)
  use <- expect.to_be_equal(bool.to_int(False), 0)

  expect.success()
}

pub fn to_string_test() {
  use <- expect.named("bool.to_string_test")

  use <- expect.to_be_equal(bool.to_string(True), "True")
  use <- expect.to_be_equal(bool.to_string(False), "False")

  expect.success()
}

pub fn guard_test() {
  use <- expect.named("bool.to_string_test")

  use <- expect.to_be_equal(
    bool.guard(when: True, return: 2, otherwise: fn() { 1 }),
    2,
  )

  use <- expect.to_be_equal(
    bool.guard(when: False, return: 2, otherwise: fn() { 1 }),
    1,
  )

  expect.success()
}

pub fn lazy_guard_test() {
  use <- expect.named("bool.lazy_guard_test")

  use <- expect.to_be_equal(
    bool.lazy_guard(when: True, return: fn() { 2 }, otherwise: fn() { 1 }),
    2,
  )

  use <- expect.to_be_equal(
    bool.lazy_guard(when: False, return: fn() { 2 }, otherwise: fn() { 1 }),
    1,
  )

  expect.success()
}
