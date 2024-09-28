(define gleam.True #t)
(define gleam.True?
  (lambda (value)
    (eq? value gleam.True)))

(define gleam.False #f)
(define gleam.False?
  (lambda (value)
    (eq? value gleam.False)))

(define gleam.Nil 'Nil)
(define gleam.Nil?
  (lambda (value)
    (eq? value gleam.Nil)))
