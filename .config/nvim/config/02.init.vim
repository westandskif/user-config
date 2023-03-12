function! TweakHighlights()
    hi! SpellBad cterm=underline gui=underline ctermfg=243 guifg=#727272 guisp=none
    " hi! GhGreen cterm=NONE gui=NONE ctermfg=114 guifg=#88a970
    " hi! GhBase5 cterm=NONE gui=NONE ctermfg=114 guifg=#d5d9dd
    " hi! Normal cterm=NONE gui=NONE ctermfg=114 guifg=#d5d9dd
    " hi! link Boolean GhOrange
    " hi! link Identifier GhBase4
    " hi! link String GhGreen
    " hi! link TabLine GhBase3
    " hi! link TabLineFill GhBase0
    " hi! link TabLineSel GhBlue
    " hi! link Type GhOrange
endfunction
autocmd ColorScheme * call TweakHighlights()

colorscheme sonokai


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
syntax on
set pumheight=20
set redrawtime=2000
set clipboard=unnamedplus
let g:netrw_altv = 1
let g:netrw_alto = 1

let g:tagbar_foldlevel = 0
let g:tagbar_autoclose = 1
let g:tagbar_sort = 0

let g:context_enabled = 0
let g:context_presenter = 'preview'

let g:qf_modifiable = 1
let g:qf_join_changes = 1
let g:qf_write_changes = 0
