(library (gleam_stdlib_ffi)
  (export
    gleam_stdlib_ffi.print
    gleam_stdlib_ffi.print_error
    gleam_stdlib_ffi.println
    gleam_stdlib_ffi.println_error
    gleam_stdlib_ffi.inspect
    gleam_stdlib_ffi.pop_grapheme)
  (import (chezscheme) (gleam/$prelude))
  
  (define gleam_stdlib_ffi.print
    (lambda (term)
      (let ((port (console-output-port))) 
        (display term port)
        gleam.Nil)))

  (define gleam_stdlib_ffi.print_error
    (lambda (term)
      (let ((port (console-error-port))) 
        (display term port)
        gleam.Nil)))

  (define gleam_stdlib_ffi.println
    (lambda (term)
      (let ((port (console-output-port))) 
        (display term port)
        (newline port)
        gleam.Nil)))

  (define gleam_stdlib_ffi.println_error
    (lambda (term)
      (let ((port (console-error-port))) 
        (display term port)
        (newline port)
        gleam.Nil)))
         
  (define gleam_stdlib_ffi.inspect
    (lambda (term)
      (with-output-to-string
        (lambda () (write term)))))

  (define gleam_stdlib_ffi.pop_grapheme
    (lambda (str)
      (if (string=? str "")
          (gleam.Error gleam.Nil)
          (let loop ((chars (string->list str)) (state 0) (current '()))
            (if (null? chars)
                (values (list->string (reverse current)) "")  ; End of string
                (let-values (((is-boundary? new-state) (char-grapheme-step (car chars) state)))
                  (if (and is-boundary? (not (null? current)))
                      (gleam.Ok (vector (list->string (reverse current)) (list->string chars)))
                      (loop (cdr chars) new-state (cons (car chars) current))))))))))
 