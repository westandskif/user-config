function! s:build_quickfix_list(lines)
  call setqflist(map(copy(a:lines), '{ "filename": v:val }'))
  copen
endfunction

let g:fzf_action = {
  \ 'ctrl-q': function('s:build_quickfix_list'),
  \ 'ctrl-t': 'tab split',
  \ 'ctrl-x': 'split',
  \ 'ctrl-v': 'vsplit' }

let $FZF_DEFAULT_OPTS="--bind ctrl-a:select-all,ctrl-f:preview-page-down,ctrl-b:preview-page-up --ansi"
let $FZF_DEFAULT_COMMAND='fd --type f --ignore-file=.ignore --exclude=__pycache__ --exclude=uploaded_files --exclude="ui/dist" --exclude="build/"'
let g:fzf_preview_window = ['right,50%', '?']

function! SetTags()
    let sitepackages = systemlist('echo $TAG_DIRS')
    for path in sitepackages
        if stridx(&tags, path) == -1
            let &tags = &tags . "," . path."/tags"
        endif
    endfor
    return sitepackages
endfunction
autocmd VimEnter * :call SetTags()
