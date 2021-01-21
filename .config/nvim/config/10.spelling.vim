" Spelling
augroup SpellingUpdateGroup
    autocmd!
	autocmd BufEnter,BufWrite,WinEnter,CursorHold *.py,*.vim,*.html,*.js,*.rs SpellingUpdate
augroup END
