lua <<EOF
require'nvim-treesitter.configs'.setup {
  ensure_installed = { "vim", "rust", "python" },
  highlight = {
    enable = true,
    additional_vim_regex_highlighting = false,
    -- disable = { "c", "rust" },  -- list of language that will be disabled
  },
  indent = { enable = false }, -- EXPERIMENTAL

  yati = {
    enable = true,
    -- Disable by languages, see `Supported languages`
    -- disable = { "python" },

    -- Whether to enable lazy mode (recommend to enable this if bad indent happens frequently)
    -- default_lazy = true,

    -- Determine the fallback method used when we cannot calculate indent by tree-sitter
    --   "auto": fallback to vim auto indent
    --   "asis": use current indent as-is
    --   "cindent": see `:h cindent()`
    -- Or a custom function return the final indent result.
    -- default_fallback = "auto"
  },

}
EOF
