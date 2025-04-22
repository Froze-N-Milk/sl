(use-modules
  (guix build-system cargo)
  (guix packages)
  (guix gexp)
  ((guix licenses) #:prefix licenses:))

(package
  (name "sl")
  (version "0")
  (source
    (file-union
      "sl"
      `(("src" ,(local-file "./src" #:recursive? #t))
	("Cargo.toml" ,(local-file "./Cargo.toml")))))
  (build-system cargo-build-system)
  (synopsis "my own little lisp")
  (description "a pretty mid lisp interpreter")
  (license licenses:expat)
  (home-page ""))
