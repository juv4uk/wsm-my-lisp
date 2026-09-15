(def process-ok?
  (lambda (cmd)
    (eq (car (process-run "bash" (list "-c" cmd))) 0)))

(def process-out
  (lambda (cmd)
    (second (process-run "bash" (list "-c" cmd)))))

(def armed-violation
  (lambda (report)
    (write-file ".guard-report.txt" report)
    (violation "WITNESS-FAIL: verification failed — see .guard-report.txt")))

(def build-witness
  (lambda (name)
    (let ((cmd1 "as --64 -o /tmp/nucleus.o /home/agents/GitHub/wsm-my-lisp/asm/nucleus.s && "))
      (let ((cmd2 (string-append "as --64 -o /tmp/entry.o /home/agents/GitHub/wsm-my-lisp/asm/entry-" name)))
        (let ((cmd3 (string-append cmd2 ".s && as --64 -o /tmp/boot.o /home/agents/GitHub/wsm-my-lisp/asm/witness-bootstrap.s && "))
              (cmd4 (string-append "ld -o /tmp/witness-" (string-append name " /tmp/nucleus.o /tmp/entry.o /tmp/boot.o"))))
          (let ((full-cmd (string-append cmd1 (string-append cmd3 cmd4))))
            (cond
              ((process-ok? full-cmd)
                (princ (string-append "BUILD OK: " (string-append name "\n"))))
              (t
                (armed-violation (string-append "BUILD FAIL: " name))))))))))

(def run-witness
  (lambda (name)
    (process-out (string-append
      "/tmp/witness-" (string-append name " 2>/dev/null | od -An -tx8 | tr -d ' \\n'")))))

(def verify
  (lambda (name expr expected)
    (princ (string-append "NAME: " (string-append name "\n")))
    (build-witness name)
    (princ (string-append "RAW: " (string-append (run-witness name) "\n")))))

(princ "=== LISP-FIRST WITNESS RUNNER (vertical slice) ===\n")
(princ "Builds via as+ld, runs ELF, captures raw word.\n")

(verify "atom" "(atom (quote ()))" "fffffffffffffffc")

(princ "=== VERTICAL SLICE COMPLETE ===\n")
()