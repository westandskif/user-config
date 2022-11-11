"========== GENERAL ==========
nnoremap <esc> :let @/=""<cr>
inoremap jk <esc>
inoremap Jk <esc>
inoremap jK <esc>
inoremap JK <esc>
tnoremap jk <C-\><C-n>
inoremap 0p <c-r>"
inoremap 0P <c-r>0
nnoremap gd :lua vim.lsp.buf.definition()<CR>

" inoremap <expr> <CR> pumvisible() ? "\<C-y>" : "\<CR>"
" inoremap <Tab> <C-n>
" inoremap <silent><expr> <esc> pumvisible() ? "\<C-n>" : "\<C-\>\<C-O>:call ncm2#manual_trigger()\<CR>"

inoremap <Tab> <C-n>
inoremap <silent><expr> <CR> pumvisible() ? compe#confirm('<CR>') : "\<CR>"
inoremap <silent><expr> <esc> pumvisible() ? compe#close('<C-e>') : compe#complete()

onoremap il :<c-u>normal! _vg_<cr>
vnoremap P "0p
vnoremap il :<c-u>normal! _vg_<cr>
nnoremap <F1> <c-y>
nnoremap <F2> <c-e>
nnoremap <F3> <c-u>
nnoremap <F4> <c-d>
nnoremap <F5> zH
nnoremap <F6> zL
vnoremap <F1> <c-y>
vnoremap <F2> <c-e>
vnoremap <F3> <c-u>
vnoremap <F4> <c-d>
vnoremap <F5> zH
vnoremap <F6> zL

command! W :w
command! Q :q

" move between windows
nnoremap gh <c-w>h
nnoremap gl <c-w>l
nnoremap gj <c-w>j
nnoremap gk <c-w>k
nnoremap gH mW<c-w>h`Wzz
nnoremap gL mW<c-w>l`Wzz
nnoremap gJ mW<c-w>j`Wzz
nnoremap gK mW<c-w>k`Wzz
function! NetrwMapping()
    nnoremap <buffer> gh <c-w>h
    nnoremap <buffer> gl <c-w>l
    nnoremap <buffer> gj <c-w>j
    nnoremap <buffer> gk <c-w>k
    nmap <buffer> - <Plug>NetrwBrowseUpDir
endfunction
augroup netrw_mapping
    autocmd!
    autocmd filetype netrw call NetrwMapping()
augroup END

let g:inflector_mapping = 'gI'

" ========== LEADER ==========
let mapleader = " "
let maplocalleader = " "
let g:which_key_map =  {}

let g:which_key_map.v = { 'name' : '☰ NVIM' }
let g:which_key_map.v.e = 'Edit configs'
let g:which_key_map.v.s = 'Source configs'
let g:which_key_map.v.q = 'Quit'
let g:which_key_map.v.C = 'Cache session'
let g:which_key_map.v.Q = 'Quit and make session'
let g:which_key_map.v.R = 'Restore session'
nnoremap <leader>ve :tabnew ~/.config/nvim/config<cr>
nnoremap <leader>vs :source ~/.config/nvim/init.vim<cr>
nnoremap <leader>vc :mksession! .s.vim<cr>
nnoremap <leader>vq :mksession! .s.vim<cr>:qa<cr>
nnoremap <leader>vr :source .s.vim<cr>


let g:which_key_map.m = {'name': '☰ MODE'}
let g:which_key_map.m.t = 'tabbar toggle'
nnoremap <leader>mt :TagbarToggle<CR>


function! IsLocListOpen()
  return len(filter(getwininfo(), 'v:val.quickfix && v:val.loclist'))
endfunction
function! PopulateQf(cmd)
  call setqflist([], ' ', {'lines' : systemlist(a:cmd), 'efm':'%f'})
  copen
  return
endfunction
command! -nargs=1 CommandToQf :call PopulateQf('<args>')

let g:which_key_map.q = { 'name' : '☰ QUICKFIX / LOCLIST' }
let g:which_key_map.q.C = 'copy loclist to quickfix'
let g:which_key_map.q.q = 'quit'
let g:which_key_map.q.D = 'populate from git Diff (adjustable command)'
nnoremap <leader>qD :CommandToQf git diff  --name-only --diff-filter=AM master...
nnoremap <silent> <leader>qC :call setqflist(getloclist(winnr()))<CR>:lclose<CR>:copen<CR>
nnoremap <silent><expr> <leader>qq IsLocListOpen() ? ":lclose\<CR>" : ":cclose\<CR>"
" go current
nnoremap <silent><expr> gc IsLocListOpen() ? ":ll\<CR>zz" : ":cc\<CR>zz"
" go next
nnoremap <silent><expr> gn IsLocListOpen() ? ":lnext\<CR>zz" : ":cnext\<CR>zz"
" go Next file
nnoremap <silent><expr> gN IsLocListOpen() ? ":lnfile\<CR>zz" : ":cnfile\<CR>zz"
" go prev
nnoremap <silent><expr> gp IsLocListOpen() ? ":lprevious\<CR>zz" : ":cprevious\<CR>zz"
" go Prev file
nnoremap <silent><expr> gP IsLocListOpen() ? ":lpfile\<CR>zz" : ":cpfile\<CR>zz"


let g:which_key_map.t = { 'name' : '☰ TAB' }
let g:which_key_map.t.n = 'new'
let g:which_key_map.t.m = 'move'
let g:which_key_map.t.q = 'quit'
nnoremap <leader>tn :tabnew<CR>
nnoremap <leader>tm :tabmove<space>
nnoremap <leader>tq :tabclose<CR>

let g:which_key_map.j = { 'name' : '☰ JUMP' }
let g:which_key_map.j.c = 'Current buffer dir'
let g:which_key_map.j.n = 'Notes'
let g:which_key_map.j.r = 'Root dir'
nnoremap <leader>jc :e%:p:h<cr>
nnoremap <leader>jn :tabnew .notes<CR>
nnoremap <leader>jr :e.<cr>


function! InputStr(prompt_str)
    call inputsave()
    let input_str =input(a:prompt_str)
    call inputrestore()
    let @i = input_str
    return input_str
endfunction
function! VimEscape(str, symbols_to_escape)
    let _symbols_to_escape = a:symbols_to_escape
    let str = a:str
    " single quote is the exception
    if stridx(_symbols_to_escape, "'") != -1
        let _symbols_to_escape = substitute(_symbols_to_escape, "'", "", "g")
        let str = substitute(a:str, "'", "''", "g")
    endif
    return escape(str, _symbols_to_escape)
endfunction
function! HistAddAndReturn(command_str)
    call histadd("cmd", a:command_str)
    return a:command_str
endfunction

function! RunRgWithOpts(command_suffix)
    let command = 'rg --column --line-number --no-heading --hidden --color=always --sort=path ' . a:command_suffix
    let fzf_args = [
                \command,
                \1,
                \fzf#vim#with_preview({'options': [
                    \'--info=inline',
                    \'--preview-window',
                    \'right:60%',
                    \'--bind',
                    \'?:toggle-preview',
                    \'--preview',
                    \'bat --color=always --style=header,grid --line-range :300 {}'
                \]}),
                \0]
    return call('fzf#vim#grep', fzf_args)
endfunction

function! VisualToRegI()
  let sel_save = &selection
  let &selection = "inclusive"

  silent exe "normal! `<".visualmode()."`>\"iy"
  let @i=substitute(@i, '\n+$', '', '')

  let &selection = sel_save
endfunction

command! -bang -nargs=+ -complete=dir Rg call RunRgWithOpts(<q-args>)


let g:which_key_map.s = { 'name' : '☰ SEARCH' }
let g:which_key_map.s.P = '"+" globally'
let g:which_key_map.s.b = 'buffers'
let g:which_key_map.s.c = 'custom Rg opts'
let g:which_key_map.s.e = 'exact'
let g:which_key_map.s.f = 'files'
let g:which_key_map.s.h = 'history'
let g:which_key_map.s.l = 'exact Locally'
let g:which_key_map.s.p = '"+" locally'
let g:which_key_map.s.r = 'Rg regex with PCRE2'
let g:which_key_map.s.t = 'tags'
let g:which_key_map.s.w = 'exact words'

let g:sym_to_escape_for_rg = "\\$#%\"'`"
let g:sym_to_escape_for_rg_regex = "#%\"'"
let g:sym_to_escape_for_buffer_search = ".* \\/[]~$^"
let g:sym_to_escape_for_buffer_search = "/\\"
nnoremap <silent><leader>sb :Buffers<cr>
nnoremap <silent><leader>sf :FZF<cr>
nnoremap <silent><leader>sh :History<cr>
nnoremap <silent><leader>st :Tags<cr>
nnoremap <leader>sl :call InputStr("exact: ")<cr>:let @i=VimEscape(@i, g:sym_to_escape_for_buffer_search)<cr>/\C\V<c-r>i
vnoremap <leader>sl :<c-u>call VisualToRegI()<cr>:let @i=VimEscape(@i, g:sym_to_escape_for_buffer_search)<cr>/\C\V<c-r>i
nnoremap <leader>sp /<c-r>=VimEscape(substitute(@+, '\n\+$', '', ''), g:sym_to_escape_for_buffer_search)<cr>
nnoremap <silent><leader>sP :let @i=VimEscape(@+, g:sym_to_escape_for_rg)<cr>:<c-r>=HistAddAndReturn('Rg --no-ignore-vcs -F -- "<c-r>i"')<cr><cr>
nnoremap <leader>sc :call InputStr("search: ")<cr>:let @i=VimEscape(@i, g:sym_to_escape_for_rg)<cr>:<c-r>=HistAddAndReturn('Rg --no-ignore-vcs -S -- "<c-r>i"')<cr>
vnoremap <leader>sc :<c-u>call VisualToRegI() <cr>:let @i=VimEscape(@i, g:sym_to_escape_for_rg)<cr>:<c-r>=HistAddAndReturn('Rg --no-ignore-vcs -S -- "<c-r>i"')<cr>
nnoremap <leader>sr :call InputStr("regex: ") <cr>:let @i=VimEscape(@i, g:sym_to_escape_for_rg_regex)<cr>:<c-r>=HistAddAndReturn('Rg --no-ignore-vcs --pcre2 -- "<c-r>i"')<cr>
vnoremap <leader>sr :<c-u>call VisualToRegI() <cr>:let @i=VimEscape(@i, g:sym_to_escape_for_rg_regex)<cr>:<c-r>=HistAddAndReturn('Rg --no-ignore-vcs --pcre2 -- "<c-r>i"')<cr>
nnoremap <silent><leader>se :call InputStr(" exact: ")<cr>:let @i=VimEscape(@i, g:sym_to_escape_for_rg)<cr>:<c-r>=HistAddAndReturn('Rg --no-ignore-vcs -F -- "<c-r>i"')<cr><cr>
vnoremap <silent><leader>se :<c-u>call VisualToRegI() <cr>:let @i=VimEscape(@i, g:sym_to_escape_for_rg)<cr>:<c-r>=HistAddAndReturn('Rg --no-ignore-vcs -F -- "<c-r>i"')<cr><cr>
nnoremap <silent><leader>sw :call InputStr(" words: ")<cr>:let @i=VimEscape(@i, g:sym_to_escape_for_rg)<cr>:<c-r>=HistAddAndReturn('Rg --no-ignore-vcs -F -w -- "<c-r>i"')<cr><cr>
vnoremap <silent><leader>sw :<c-u>call VisualToRegI() <cr>:let @i=VimEscape(@i, g:sym_to_escape_for_rg)<cr>:<c-r>=HistAddAndReturn('Rg --no-ignore-vcs -F -w -- "<c-r>i"')<cr><cr>



let g:which_key_map.l = { 'name' : '☰ LANGUAGE' }
" =================================================

let g:which_key_map.l.d = 'go to definition'
let g:which_key_map.l.m = { 'name' : '☰ Make' }
let g:which_key_map.l.m.q = 'Quick'
let g:which_key_map.l.m.a = 'All'
let g:which_key_map.l.m.Q = { 'name' : '☰ Quickfix' }
let g:which_key_map.l.m.Q.Q = 'Quick'
let g:which_key_map.l.m.Q.A = 'All'

nnoremap <leader>ld :call LanguageClient#textDocument_definition()<CR>
nnoremap <leader>lh :call LanguageClient#textDocument_hover()<CR>
nnoremap <leader>lma :Neomake<cr>
nnoremap <leader>lmQQ :call NeomakeQf(neomake_qf_lint_quick)<CR>
nnoremap <leader>lmQA :call NeomakeQf(neomake_qf_lint_full)<CR>

let g:which_key_map.l.a = { 'name' : '☰ Add' }
let g:which_key_map.l.a.i = 'import'
let g:which_key_map.l.a.w = 'word'
nnoremap <leader>law :SpellingAddWord<cr>

let g:which_key_map.l.t = { 'name' : '☰ Toggle' }
let g:which_key_map.l.t.c = 'toggle context'
let g:which_key_map.l.t.s = 'toggle spelling'
nnoremap <leader>ltc :ContextToggle<CR>
nnoremap <leader>lts :SpellingToggle<CR>

let g:which_key_map.l.f = { 'name': '☰ Fix' }
let g:which_key_map.l.f.a = 'all format'
let g:which_key_map.l.f.f = 'format'
let g:which_key_map.l.f.i = 'imports'
nnoremap <leader>lfa :Neoformat<CR>
nnoremap <leader>lff :Neoformat<CR>
nnoremap <leader>lfi :Neoformat<CR>


let g:which_key_map.b = {'name': '☰ BUFFERS'}
let g:which_key_map.b.q = 'quit all'
nnoremap <silent> <leader>bq :Bdelete hidden<CR>

let g:which_key_map.c = {'name': '☰ COMMANDS'}
let g:which_key_map.c.f = 'copy file'
let g:which_key_map.c.p = 'copy file path'
let g:which_key_map.c.t = 'Trim whitespaces'
let g:which_key_map.c.R = 'Refresh buffers & regenerate local tags'
let g:which_key_map.c.T = 'refresh outer Tags'
nnoremap <leader>cR :syntax off<CR>:let _current_buffer=bufnr("%")<CR>:bufdo execute ":e"<CR>:b <c-r>=_current_buffer<CR><CR>:GutentagsUpdate!<CR>:syntax on<CR>
nnoremap <leader>cf :!cp '%:p' '%:p:h/.%:e'<Left><Left><Left><Left><Left>
nnoremap <leader>cT :call GenerateSitePackageTags()<CR>
nnoremap <silent> <leader>cp :let @0=@%<CR>:let @+=@%<CR>
nnoremap <silent> <leader>ct :%s/\s\+$//g<CR>

call which_key#register('<Space>', 'g:which_key_map')
nnoremap <silent> <leader> :<c-u>WhichKey '<Space>'<CR>
vnoremap <silent> <leader> :<c-u>WhichKeyVisual '<Space>'<CR>

