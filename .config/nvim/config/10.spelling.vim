" Spelling
augroup SpellingUpdateGroup
    autocmd!
	autocmd BufEnter,BufWrite,WinEnter,CursorHold *.py,*.vim,*.html,*.js SpellingUpdate
augroup END
