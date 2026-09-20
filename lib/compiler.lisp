; Bootstrap compiler seed: Lisp source form -> CML-shaped IR as Lisp data.
;
; This deliberately starts with a small CML-shaped vocabulary:
;   (quote datum)       -> (quote datum)
;   (callee arg ...)    -> (app callee-ir (args-ir ...))
;   (lambda (x) body)   -> (lambda (x) body-ir)
;   (cond ...)           -> (cond-match ...)
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

(def compiler-head?
  (lambda (form name)
    (cond
      ((atom form) (quote ()))
      (t (eq (car form) name)))))

(def compiler-clause
  (lambda (clause)
    (list (compiler-form (car clause))
          (car (cdr clause))
          (compiler-form (car (cdr (cdr clause)))))))

(def compiler-clauses
  (lambda (clauses)
    (cond
      ((atom clauses) (quote ()))
      (t (cons (compiler-clause (car clauses))
               (compiler-clauses (cdr clauses)))))))

(def compiler-form
  (lambda (form)
    (cond
      ((compiler-quote? form)
       (list (quote quote) (car (cdr form))))
      ((compiler-head? form (quote lambda))
       (list (quote lambda)
             (car (cdr form))
             (compiler-form (car (cdr (cdr form))))))
      ((compiler-head? form (quote cond))
       (list (quote cond-match)
             (compiler-clauses (cdr form))))
      ((atom form)
       (list (quote var) form))
      (t
       (list (quote app)
             (compiler-form (car form))
             (compiler-map compiler-form (cdr form)))))))

(def compiler-program
  (lambda (forms)
    (compiler-map compiler-form forms)))

(quote compiler-bootstrap-seed)
