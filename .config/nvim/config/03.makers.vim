let g:neoformat_run_all_formatters = 1
let g:neoformat_enabled_python = ['isort', 'black']
let g:neoformat_hcl_hclfmt = {'exe': 'hclfmt'}
let g:neoformat_enabled_hcl = ['hclfmt']

let s:denoFormatter = {
      \ 'exe': 'deno',
      \ 'args': ['fmt', '-'],
      \ 'stdin': 1,
      \ }

let g:neoformat_javascript_deno = s:denoFormatter
let g:neoformat_enabled_javascript = ['deno']
let g:neoformat_typescript_deno = s:denoFormatter
let g:neoformat_enabled_typescript = ['deno']
let g:neoformat_typescriptreact_deno = s:denoFormatter
let g:neoformat_enabled_typescriptreact = ['deno']
