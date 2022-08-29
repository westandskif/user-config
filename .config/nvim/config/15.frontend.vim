function FrontendSpecifics()
  nnoremap <buffer> <localleader>lff :Neoformat prettier<CR>
endfunction

autocmd FileType html,javascript,typescriptreact,less :call FrontendSpecifics()
