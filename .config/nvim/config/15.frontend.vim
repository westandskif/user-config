function FrontendSpecifics()
  nnoremap <buffer> <localleader>lff :Neoformat prettier<CR>
endfunction
autocmd FileType html,javascript,typescriptreact,less :call FrontendSpecifics()

function TsSpecifics()
  nnoremap <buffer> <localleader>lff :Neoformat tsfmt<CR>
endfunction
autocmd FileType typescript :call TsSpecifics()
