"========== GENERAL ==========
nnoremap <esc> :let @/=""<cr>
inoremap jk <esc>
tnoremap jk <C-\><C-n>
inoremap Pp <c-r>"
inoremap PP <c-r>0


inoremap <expr> <CR> pumvisible() ? "\<C-y>" : "\<CR>"
inoremap <Tab> <C-n>
inoremap <silent><expr> <esc> pumvisible() ? "\<C-n>" : "\<C-\>\<C-O>:call ncm2#manual_trigger()\<CR>"

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
nnoremap <leader>vq :qa<cr>
nnoremap <leader>vC :mksession! .s.vim<cr>
nnoremap <leader>vQ :mksession! .s.vim<cr>:qa<cr>
nnoremap <leader>vR :source .s.vim<cr>


let g:which_key_map.m = {'name': '☰ MODE'}
let g:which_key_map.m.t = 'tabbar toggle'
nnoremap <leader>mt :TagbarToggle<CR>


function IsQuickfixOpen()
  return len(filter(getwininfo(), 'v:val.quickfix && !v:val.loclist'))
endfunction

let g:which_key_map.q = { 'name' : '☰ QUICKFIX / LOCLIST' }
let g:which_key_map.q.o = {'name': '☰ Open'}
let g:which_key_map.q.c = 'current'
let g:which_key_map.q.C = 'copy loclist to quickfix'
let g:which_key_map.q.n = 'next'
let g:which_key_map.q.o.l = 'loclist'
let g:which_key_map.q.o.q = 'quickfix'
let g:which_key_map.q.p = 'previous'
let g:which_key_map.q.q = 'quit'
let g:which_key_map.q.f = 'Filter lines with substr'
let g:which_key_map.q.e = 'Exclude lines with substr'
let g:which_key_map.q.d = 'populate from git Diff'
nnoremap <silent> <leader>qol  :lopen<CR>
nnoremap <silent> <leader>qoq  :copen<CR>
nnoremap <silent> <leader>qC :call setqflist(getloclist(winnr()))<CR>
nnoremap <silent><expr> <leader>qc IsQuickfixOpen() ? ":cc\<CR>" : ":ll\<CR>"
nnoremap <silent><expr> <leader>qn IsQuickfixOpen() ? ":cnext\<CR>" : ":lnext\<CR>"
nnoremap <silent><expr> <leader>qp IsQuickfixOpen() ? ":cprevious\<CR>" : ":lprevious\<CR>"
nnoremap <silent><expr> <leader>qq IsQuickfixOpen() ? ":cclose\<CR>" : ":lclose\<CR>"

nnoremap gh <c-w>h
nnoremap gl <c-w>l
nnoremap gj <c-w>j
nnoremap gk <c-w>k
augroup netrw_mapping
    autocmd!
    autocmd filetype netrw call NetrwMapping()
augroup END

function! NetrwMapping()
    nnoremap <buffer> gh <c-w>h
    nnoremap <buffer> gl <c-w>l
    nnoremap <buffer> gj <c-w>j
    nnoremap <buffer> gk <c-w>k
endfunction

let g:which_key_map.t = { 'name' : '☰ TAB' }
let g:which_key_map.t.n = 'new'
let g:which_key_map.t.m = 'move'
let g:which_key_map.t.q = 'quit'
nnoremap <leader>tn :tabnew<CR>
nnoremap <leader>tm :tabmove<space>
nnoremap <leader>tq :tabclose<CR>

let g:which_key_map.j = { 'name' : '☰ JUMP' }
let g:which_key_map.j.c = 'current buffer dir'
let g:which_key_map.j.n = 'open notes'
let g:which_key_map.j.r = 'pwd'
nnoremap <leader>jc :e%:p:h<cr>
nnoremap <leader>jn :tabnew .notes<CR>
nnoremap <leader>jr :e.<cr>

let g:which_key_map.s = { 'name' : '☰ SEARCH' }
let g:which_key_map.s.b = 'buffers'
let g:which_key_map.s.e = 'exact'
let g:which_key_map.s.f = 'files'
let g:which_key_map.s.h = 'history'
let g:which_key_map.s.s = 'exact search selection'
let g:which_key_map.s.c = 'custom Rg opts'
let g:which_key_map.s.t = 'tags'
let g:which_key_map.s.w = 'exact words'
let g:which_key_map.s.p = '"+" locally'
let g:which_key_map.s.P = '"+" globally'

function InputStr(prompt_str)
    call inputsave()
    let l:input_str =input(a:prompt_str)
    call inputrestore()
    let @i = l:input_str
    return l:input_str
endfunction
function VimEscape(str, symbols_to_escape)
    let l:_symbols_to_escape = a:symbols_to_escape
    let l:str = a:str
    " single quote is the exception
    if stridx(l:_symbols_to_escape, "'") != -1
        let l:_symbols_to_escape = substitute(l:_symbols_to_escape, "'", "", "g")
        let l:str = substitute(a:str, "'", "''", "g")
    endif
    return escape(l:str, l:_symbols_to_escape)
endfunction
function HistAddAndReturn(command_str)
    :call histadd("cmd", a:command_str)
    return a:command_str
endfunction

function RunRgWithOpts(command_suffix)
    let l:command = 'rg --column --line-number --no-heading --color=always --sort=path ' . a:command_suffix
    let l:fzf_args = [
                \l:command,
                \1,
                \fzf#vim#with_preview({'options': [
                    \'--info=inline',
                    \'--preview-window',
                    \'right:50%',
                    \'--preview',
                    \'bat --color=always --style=header,grid --line-range :300 {}'
                \]}),
                \0]
    return call('fzf#vim#grep', l:fzf_args)
endfunction

" FOR <leader>ss binding to work with vim movements
function! RgExactSearch(type, ...)
  let sel_save = &selection
  let &selection = "inclusive"
  let reg_save = @@

  if a:0  " Invoked from Visual mode, use '< and '> marks.
    silent exe "normal! `<" . a:type . "`>y"
  elseif a:type == 'line'
    silent exe "normal! '[V']y"
  elseif a:type == 'block'
    silent exe "normal! `[\<C-V>`]y"
  else
    silent exe "normal! `[v`]y"
  endif

  let @i=VimEscape(@@, "\\$#%\"`")
  silent execute HistAddAndReturn('Rg "'.@i.'" --no-ignore-vcs -F')

  let &selection = sel_save
  let @@ = reg_save
endfunction

command! -bang -nargs=+ -complete=dir Rg call RunRgWithOpts(<q-args>)

let g:which_key_map.s.s = 'exact search selection'
nnoremap <silent><leader>ss :set operatorfunc=RgExactSearch<CR>g@
vnoremap <silent><leader>ss :<C-U>call RgExactSearch(visualmode(), 1)<CR>

let g:which_key_map.s.p = '"+" locally'
nnoremap <leader>sp /<c-r>=VimEscape(substitute(@+, '\n\+$', '', ''), ".* \\/[]~")<cr>
let g:which_key_map.s.P = '"+" globally'
nnoremap <silent><leader>sP :let @i=VimEscape(@+, "\\$#%\"'`")<cr>:<c-r>=HistAddAndReturn('Rg "<c-r>i" --no-ignore-vcs -F')<cr><cr>

let g:which_key_map.s.e = 'exact'
nnoremap <silent><leader>se :let @i=VimEscape(InputStr(" exact: "), "\\$#%\"'`")<cr>:<c-r>=HistAddAndReturn('Rg "<c-r>i" --no-ignore-vcs -F')<cr><cr>
let g:which_key_map.s.w = 'exact words'
nnoremap <silent><leader>sw :let @i=VimEscape(InputStr(" words: "), "\\$#%\"'`")<cr>:<c-r>=HistAddAndReturn('Rg "<c-r>i" --no-ignore-vcs -F -w')<cr><cr>

let g:which_key_map.s.c = 'custom Rg opts'
nnoremap <leader>sc :let @i=VimEscape(InputStr("search: "), "\\$#%\"'`")<cr>:<c-r>=HistAddAndReturn('Rg "<c-r>i" --no-ignore-vcs -S')<cr>


nnoremap <leader>sb :Buffers<cr>
nnoremap <leader>sf :FZF<cr>
nnoremap <leader>sh :History<cr>
nnoremap <leader>st :Tags<cr>


function s:FilterList(list, str, leave)
    if a:leave
        return filter(a:list, {idx, val -> (stridx(val.text, a:str) > -1 || stridx(bufname(val.bufnr), a:str) > -1)})
    else
        return filter(a:list, {idx, val -> (stridx(val.text, a:str) == -1 && stridx(bufname(val.bufnr), a:str) == -1)})
    endif
endfunction
function FilterQfLocList(str, leave)
  let l:qf_number = len(filter(getwininfo(), 'v:val.quickfix && !v:val.loclist'))
  if l:qf_number == 1
      let l:list = getqflist()
      call setqflist(s:FilterList(l:list, a:str, a:leave))
      return
  endif

  let l:tabnr = tabpagenr()
  let l:lists = filter(getwininfo(), {idx, val -> v:val.quickfix && v:val.loclist && l:tabnr == v:val.tabnr})
  if len(l:lists) == 1
      let l:loclist_number = l:lists[0]['loclist'] 
      let l:list = getloclist(l:loclist_number)
      call setloclist(l:loclist_number, s:FilterList(l:list, a:str, a:leave))
  endif
endfunction

nnoremap <silent><leader>qf :let @i=VimEscape(InputStr("lines to leave: "), "\\$#%\"'`")<cr>:<c-r>=HistAddAndReturn('call FilterQfLocList("<c-r>i", 1)')<cr><cr>
nnoremap <silent><leader>qe :let @i=VimEscape(InputStr("lines to exclude: "), "\\$#%\"'`")<cr>:<c-r>=HistAddAndReturn('call FilterQfLocList("<c-r>i", 0)')<cr><cr>
nnoremap <silent><leader>qd :call setqflist([], ' ', {'lines' : systemlist('git diff --name-only --cached --diff-filter=AM'), 'efm':'%f'})<cr>:copen<cr>


let g:which_key_map.l = { 'name' : '☰ LANGUAGE' }
let g:which_key_map.l.d = 'go to definition'
let g:which_key_map.l.m = { 'name' : '☰ Make' }
let g:which_key_map.l.m.a = 'All'
let g:which_key_map.l.m.q = 'Quick'
let g:which_key_map.l.m.d = 'git Diff'
" ALSO nnoremap gd IS DEFINED LOCALLY FOR BUFFERS
nnoremap <leader>ld :call LanguageClient#textDocument_definition()<CR>
nnoremap <leader>lma :Neomake<cr>
nnoremap <leader>lmd :call NeomakeGitDiff()<CR>

let g:which_key_map.l.a = { 'name' : '☰ Add' }
let g:which_key_map.l.a.b = 'breakpoint'
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

let g:which_key_map.l.o = { 'name': '☰ Optimize' }
let g:which_key_map.l.o.i = 'imports'

let g:which_key_map.l.s = { 'name': '☰ Spelling' }
let g:which_key_map.l.s.a = 'Add word'
nnoremap <leader>lsa :SpellingAddWord<CR>

let g:which_key_map.b = {'name': '☰ BUFFERS'}
nnoremap <silent> <leader>bq :Bdelete hidden<CR>
let g:which_key_map.b.q = 'quit all'

let g:which_key_map.c = {'name': '☰ COMMANDS'}
let g:which_key_map.c.f = 'copy file'
let g:which_key_map.c.p = 'copy file path'
let g:which_key_map.c.t = 'Trim whitespaces'
let g:which_key_map.c.R = 'Refresh buffers & regenerate local tags'
let g:which_key_map.c.T = 'Refresh outer tags'
nnoremap <leader>cR :syntax off<CR>:let _current_buffer=bufnr("%")<CR>:bufdo execute ":e"<CR>:b <c-r>=_current_buffer<CR><CR>:GutentagsUpdate!<CR>:syntax on<CR>
nnoremap <leader>cf :!cp '%:p' '%:p:h/.%:e'<Left><Left><Left><Left><Left>
nnoremap <leader>cT :call GenerateSitePackageTags()<CR>
nnoremap <silent> <leader>cp :let @0=@%<CR>:let @+=@%<CR>
nnoremap <silent> <leader>ct :%s/\s\+$//g<CR>


call which_key#register('<Space>', "g:which_key_map")
nnoremap <silent> <leader> :<c-u>WhichKey '<Space>'<CR>
vnoremap <silent> <leader> :<c-u>WhichKeyVisual '<Space>'<CR>

