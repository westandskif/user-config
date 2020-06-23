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
let $FZF_DEFAULT_COMMAND='fd --type f --no-ignore --exclude=__pycache__ --exclude=uploaded_files --exclude="ui/dist" --exclude="build/"'
