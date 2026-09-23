; Bootstrap compiler seed: Lisp source form -> CML-shaped IR as Lisp data.
;
; This deliberately starts with a small CML-shaped vocabulary:
;   (quote datum)       -> (quote datum)
;   primitive call      -> (prim SID8 (args-ir ...))
;   (callee arg ...)    -> (app callee-ir (args-ir ...))
;   (lambda (x) body)   -> (lambda (x) body-ir)
;   (cond ...)           -> (cond ...)
;
; Function resolution is one-way: source spellings are compared only while
; recognizing the input form. Once a primitive is resolved, the emitted
; function-key field is the exact bare SID8 runtime value, never a quoted
; symbol/string/name alias.
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

(def compiler-primitive-sid
  (lambda (name)
    (cond
      ((eq name (quote atom)) 00000010)
      ((eq name (quote eq)) 00000011)
      ((eq name (quote cons)) 00000100)
      ((eq name (quote car)) 00000101)
      ((eq name (quote cdr)) 00000110)
      ((eq name (quote +)) 00001100)
      ((eq name (quote -)) 00001101)
      (t (quote ())))))

(def compiler-primitive?
  (lambda (name)
    (cond
      ((compiler-primitive-sid name) t)
      (t (quote ())))))

(def compiler-nil?
  (lambda (form)
    (cond
      ((atom form) (eq form (quote ())))
      (t (quote ())))))

(def compiler-clause
  (lambda (clause)
    (cond
      ((compiler-clause-shape? clause)
       (list (compiler-form (car clause))
             (compiler-form (car (cdr clause)))))
      (t (list (quote compile-error)
               (quote malformed-cond-clause)
               clause)))))

(def compiler-clause-shape?
  (lambda (clause)
    (cond
      ((atom clause) (quote ()))
      ((atom (cdr clause)) (quote ()))
      ((not (atom (cdr (cdr clause)))) (quote ()))
      (t t))))

(def compiler-clauses
  (lambda (clauses)
    (cond
      ((atom clauses) (quote ()))
      (t (cons (compiler-clause (car clauses))
               (compiler-clauses (cdr clauses)))))))

(def compiler-form
  (lambda (form)
    (cond
      ((compiler-nil? form)
       (list (quote nil)))
      ((compiler-quote? form)
       (list (quote quote) (car (cdr form))))
      ((compiler-head? form (quote lambda))
       (list (quote lambda)
             (car (cdr form))
             (compiler-form (car (cdr (cdr form))))))
      ((compiler-head? form (quote def))
       (list (quote def)
             (car (cdr form))
             (compiler-form (car (cdr (cdr form))))))
      ((compiler-head? form (quote define))
       (list (quote def)
             (car (cdr form))
             (compiler-form (car (cdr (cdr form))))))
      ((compiler-head? form (quote cond))
       (cons (quote cond)
             (compiler-clauses (cdr form))))
      ((atom form)
       (list (quote var) form))
      ((compiler-primitive? (car form))
       (list (quote prim)
             (compiler-primitive-sid (car form))
             (compiler-map compiler-form (cdr form))))
      (t
       (list (quote app)
             (compiler-form (car form))
             (compiler-map compiler-form (cdr form)))))))

(def compiler-program
  (lambda (forms)
    (compiler-map compiler-form forms)))

(quote compiler-bootstrap-seed)
