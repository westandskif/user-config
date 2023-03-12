" pip install sqlparse
" cat tst.sql | sqlformat --reindent --indent_columns -

function SqlSpecifics()
  vnoremap <buffer> <localleader>lff :!sqlformat --reindent --indent_columns -<CR>
endfunction

autocmd FileType sql :call SqlSpecifics()
