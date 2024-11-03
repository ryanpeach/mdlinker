(module
  (tree-sitter-markdown
    (grammar
      (name "markdown")

      ;; Top-level rules
      (extras $ (token_choice "\n" " " "\t"))

      (rules
        ;; Define rule for tags like #tag or #[[tag]]
        (wikilink
          (choice
            (pattern "#[a-zA-Z0-9_-]+")     ;; Matches #wikilink
            (seq                            ;; Matches #[[wikilink]]
              "#"
              "[["
              (repeat1 (not-char "]"))
              "]]")
            (seq                            ;; Matches [[wikilink]]
              "[["
              (repeat1 (not-char "]"))
              "]]")))

        ;; Define rule for frontmatter parameter like foo:: a, b, c
        (alias
          (seq
            "alias::"    ;; Matches the parameter name with ::
            (repeat1 (seq
              (pattern "(?:\s*)[a-zA-Z0-9_-]+") ;; Matches the values in CSV format
              ",")))                            ;; CSV
        )
      )
    )
  )
)
