(define args (command-line-arguments))
(define proj (car args))

(define get-project-deps
  (lambda (project)
    (let ((deps-path (string-append project ".deps")))
      (if (file-exists? deps-path)
        (let ((port (open-input-file deps-path)))
          (do-get-project-deps port (list)))
        (list)))))

(define do-get-project-deps
  (lambda (port deps)
    (let ((dep (get-line port)))
      (cond
        ((eof-object? dep)
          deps)
        (else
          (let ((dep-deps (get-project-deps dep)))
            (do-get-project-deps port (append dep-deps (cons dep deps)))))))))

(compile-file "build/dev/chez/gleam/$prelude.scm")

(define deps (append (get-project-deps proj) (list proj (string-append proj ".main"))))
(define srcs (map (lambda (src) (string-append src ".scm")) deps))

(define compile-gleam-file
  (lambda (path)
    (if (file-exists? path)
      (compile-file path))))

(for-each compile-gleam-file srcs)
