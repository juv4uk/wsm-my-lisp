; Bootstrap compiler seed: Lisp source form -> CML-shaped IR as Lisp data.
;
; This deliberately starts with two forms only:
;   (quote datum)       -> (literal datum)
;   (callee arg ...)    -> (call callee (args ...))
;
; The result is data, not an evaluator result and not semantic authority. CML
; remains the bootstrap backend; my-lisp remains the reference oracle.

(def compiler-map
  (lambda (fn values)
    (cond
      ((atom values) (quote ()))
      (t (cons (fn (car values))
               (compiler-map fn (cdr values)))))))

(def compiler-quote?
  (lambda (form)
    (cond
      ((atom form) (quote ()))
      ((atom (car form))
       (eq (car form) (quote quote)))
      (t (quote ())))))

(def compiler-form
  (lambda (form)
    (cond
      ((compiler-quote? form)
       (list (quote literal) (car (cdr form))))
      ((atom form)
       (list (quote variable) form))
      (t
       (list (quote call)
             (compiler-form (car form))
             (compiler-map compiler-form (cdr form)))))))

(def compiler-program
  (lambda (forms)
    (compiler-map compiler-form forms)))

(quote compiler-bootstrap-seed)
