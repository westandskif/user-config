function FrontendSpecifics()
  nnoremap <buffer> <localleader>lff :Neoformat prettier<CR>
endfunction
autocmd FileType html,javascript,less :call FrontendSpecifics()

function TsSpecifics()
  nnoremap <buffer> <localleader>lff :Neoformat deno<CR>
endfunction
autocmd FileType typescript,typescriptreact :call TsSpecifics()
