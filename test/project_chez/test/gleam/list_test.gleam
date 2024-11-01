import expect
import gleam/int
import gleam/list
import gleam/string

const recursion_test_cycles = 1_000_000

pub fn length_test() {
  use <- expect.named("list.length_test")

  use <- expect.to_be_equal(list.length([]), 0)
  use <- expect.to_be_equal(list.length([1]), 1)
  use <- expect.to_be_equal(list.length([1, 1]), 2)
  use <- expect.to_be_equal(list.length([1, 1, 1]), 3)

  list.range(0, recursion_test_cycles)
  |> list.length()

  expect.success()
}

pub fn count_test() {
  use <- expect.named("list.count_test")

  use <- expect.to_be_equal(list.count([], int.is_odd), 0)
  use <- expect.to_be_equal(list.count([2, 4, 6], int.is_odd), 0)
  use <- expect.to_be_equal(list.count([1, 2, 3, 4, 5], int.is_odd), 3)
  use <- expect.to_be_equal(
    list.count(["a", "list", "with", "some", "string", "values"], fn(a) {
      string.length(a) > 4
    }),
    2,
  )

  expect.success()
}
