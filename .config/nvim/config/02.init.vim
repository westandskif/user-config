autocmd ColorScheme * highlight SpellBad cterm=underline gui=underline ctermfg=243 guifg=#727272

colorscheme space_vim_theme
" colorscheme darcula

" Protect changes between writes. Default values of
" updatecount (200 keystrokes) and updatetime
" (4 seconds) are fine
set swapfile
set updatetime=10000
" protect against crash-during-write
set writebackup
" but do not persist backup after successful write
set nobackup
" use rename-and-write-new method whenever safe
set backupcopy=auto
" persist the undo tree for each file
set undofile


set termguicolors
set hlsearch incsearch
set number
set nowrap
set smartcase
set ignorecase
set splitright
set splitbelow

set signcolumn=yes:1
set encoding=utf-8
set laststatus=2
set statusline=%F         " Path to the file
set statusline+=%<
set statusline+=%=        " Switch to the right side
set statusline=%F         " Path to the file
set statusline+=%P        " Percentage
set statusline+=\ of\     " Separator
set statusline+=%L        " Total lines
set completeopt=noinsert,menuone,noselect
syntax on
set completeopt=menu,longest
set pumheight=20
set redrawtime=10000
set clipboard=unnamedplus
let g:netrw_altv = 1
let g:netrw_alto = 1

let g:tagbar_foldlevel = 0
let g:tagbar_autoclose = 1
let g:tagbar_sort = 0

let g:context_enabled = 0
let g:context_presenter = 'preview'
let g:loaded_sql_completion = 0
let g:python_highlight_func_calls = 0

let g:qf_modifiable = 1
let g:qf_join_changes = 1
let g:qf_write_changes = 0
