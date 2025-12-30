lua require('plugins')
lua require('init_config')
lua require('keybindings')
lua require('python')
lua require('search')
lua require('lualine_config')
lua require('lsp')

if strlen($NVIM_PYTHON3_HOST_PROG) > 0
    let g:loaded_python_provider = 0
    let g:python3_host_prog = $NVIM_PYTHON3_HOST_PROG
endif

" for f in sort(split(glob(expand('<sfile>:p:h') . "/config/*.vim"), '\n'))
"     exe 'source' f
" endfor
