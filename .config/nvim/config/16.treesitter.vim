lua <<EOF
require'nvim-treesitter.configs'.setup {
  ensure_installed = { "vim", "rust", "python" },
  highlight = {
    enable = true,
    additional_vim_regex_highlighting = false,
    -- disable = { "c", "rust" },  -- list of language that will be disabled
  },
  indent = { enable = false }, -- EXPERIMENTAL
}
EOF
