" lua <<EOF
" require'nvim-treesitter.configs'.setup {
"     ensure_installed = { "rust", "python" },
"   -- ignore_install = { "javascript" }, -- List of parsers to ignore installing
"   highlight = {
"     enable = true,              -- false will disable the whole extension
"     -- disable = { "c", "rust" },  -- list of language that will be disabled
"   },
"   indent = { enable = true }, -- EXPERIMENTAL
" }
" EOF
