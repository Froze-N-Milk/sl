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

;;; Let:
;;; (let ((name value)...) body)
;;; binds each `value` to its `name` within `body`, evauluates body

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

;;; Begin / Define:
;;; (begin define-form... body)
;;; (define name value)
;;; (define (name args...) body)
;;;
;;; define forms are only available in a begin form
;;; they bind names to values linearly, similar to let
;;; the second define form is a short hand for
;;; (define name (lambda (args...) body))
;;;
;;; the file itself is included in a begin form,
;;; which means you can write define-forms before the result of the file

;;; If:
;;; (if? cond pass-body fail-body)
;;; evaluates cond. all values except for `#f` and the empty list `(quote ())`
;;; evaluate as truthy.
;;; if cond is truthy, evaluates pass-body
;;; else, evaluates fail-body

;;; Guard:
;;; (guard? (cond body)... fail-body)
;;; tests each branch form cond and evaluates its body if the cond is truthy.
;;; if none pass, evauluates fail-body.

;;; Data Types:
;;; procedure - function
;;; symbol - single word identifier
;;; list - list of other data types
;;; bool - boolean (#t or #f)

;;;
;;; Hello World:
;;;

(quote (hello world))
