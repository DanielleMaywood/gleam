(library (gleam/$prelude)
  (export
    gleam.True
    gleam.True?
    gleam.False
    gleam.False?
    gleam.Nil
    gleam.Nil?
    gleam.Ok
    gleam.Ok?
    gleam.Error
    gleam.Error?
    gleam.string-prefix?)
  (import (chezscheme))

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

  (define gleam.Ok
    (lambda (v)
      (list 'Ok (vector v))))

  (define gleam.Ok?
    (lambda (v)
      (eq? (car v) 'Ok)))

  (define gleam.Error
    (lambda (v)
      (list 'Error (vector v))))

  (define gleam.Error?
    (lambda (v)
      (eq? (car v) 'Error)))

  (define (gleam.string-prefix? str prefix)
    (let ((len (string-length prefix)))
      (and (>= (string-length str) len)
           (string=? (substring str 0 len) prefix)))))
