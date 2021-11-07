function FrontendSpecifics()
  nnoremap <buffer> <localleader>lff :Neoformat prettier<CR>
endfunction

autocmd FileType html,javascript,less :call FrontendSpecifics()
