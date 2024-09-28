(define project_chez_ffi.print
  (lambda (term)
    (let () (display term) gleam.Nil)))

(define project_chez_ffi.println
  (lambda (term)
    (let () (display term) (newline) gleam.Nil)))
