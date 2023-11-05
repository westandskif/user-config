function! PathAsImport(path)
    let path = substitute(a:path, '/', '.', 'g')
    let path = substitute(path, '.py$', '', '')
    return 'from '. path . ' import '
endfunction
function! s:do_import(import_path)
    execute 'normal O' . a:import_path
endfunction
function! AutoImport(name)
    " find exact tag
    let tags = taglist('^'.a:name.'$')
    " filter top-level tags
    let tags = filter(tags, "v:val['cmd'][2] != ' '")
    let matched_files = map(tags, "v:val['filename']")
    for tagfile in tagfiles()
        let base = fnamemodify(tagfile, ':p:h')
        let matched_files = map(
                    \   matched_files,
                    \   {i, x -> substitute(x, base . '/', '', '')}
                    \ )
    endfor
    " prepare import strings
    let matched_files = map(matched_files, {i, x -> PathAsImport(x).a:name})
    let size = len(matched_files)
    if size < 1
        echohl WarningMsg
        echo "[AutoImport] Not found!"
        echohl None
        return
    endif
    if size == 1
        call s:do_import(matched_files[0])
        return
    endif
    call fzf#run(fzf#wrap({
    \   'source': matched_files,
    \   'sink': function('s:do_import'),
    \ }))
endfunction

function! Noop(...)
    return v:null
endfunction

function PythonSpecifics()
  nnoremap <buffer> <localleader>lff :Neoformat black<CR>
  nnoremap <buffer> <localleader>lfi :Neoformat isort<CR>
  nnoremap <buffer> <localleader>lmq :Neomake ruff<CR>
  nnoremap <buffer> <localleader>lma :Neomake pylint<CR>
  nnoremap <buffer> <silent> <localleader>lai :call AutoImport(expand('<cword>'))<CR>

  " preventing vim.lsp.tagfunc installation
  setlocal tagfunc=Noop
endfunction

autocmd FileType python :call PythonSpecifics()
" let g:polyglot_disabled = ['python-indent']
let g:python_pep8_indent_searchpair_timeout = 20
