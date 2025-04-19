;;;
;;; Comments:
;;;
;;; the `;` indicates a comment
;;; comments continue to the end of the line
;;;
;;; rule of thumb:
;;; `;;;` for documentation
;;; `;;` for general comments
;;; `;` for end of line comments

;;;
;;; Builtins:
;;;

;;; Application:
;;; (quote x)
;;; here the procedure `quote` is applied to the single argument `x`
;;; this will run the function quote, and return the result

;;; Lambda:
;;; (lambda (args...) body)
;;; defines a procedure with arguments bound to the names in `args`
;;; when applied, binds values to the names and evaluates the expression `body`

;;; Macro (LIKELY TO BE REMOVED):
;;; (macro (arg) body)
;;; defines a macro procedure with a single argument bound to `arg`
;;; when applied, binds the list of applied arguments to a list as `arg`
;;; the majority of other builtins are implemented with macro (in rust, but
;;; using the same mechanism), its how you can customise the syntax (to some
;;; degree)

;;; Let:
;;; (let ((name value)...) body)
;;; binds each `value` to its `name` within `body`, evuluates body

;;; Quote:
;;; (quote expr)
;;; returns the symbol equivalent of expr. expr is not evaluated
;;; see [eval], [quasiquote], [unquote]

;;; Unquote:
;;; (unquote expr)
;;; unquotes an expression within [quasiquote], causing it to be evaluated,
;;; only valid within quasiquote
;;; see [eval], [quasiquote], [quote]

;;; Quasiquote:
;;; (quasiquote expr)
;;; quotes an expression, but [unquote] can be called from within to cause
;;; expressions to be selectively evaluated.
;;; see [eval], [unquote], [quote]

;;; Eval:
;;; (eval expr)
;;; evaluates an expression, which will run quoted code
;;; see [quote], [quasiquote], [unquote]

;;; Data Types:
;;; procedure / macro-procedure - essentially the same, but are different under
;;; the hood atm
;;; symbol - single word identifier
;;; list - list of other data types

;;;
;;; Hello World:
;;;

(quote hello world)
