call plug#begin('~/.local/share/nvim/plugged')

" THEMES
" https://github.com/rockerBOO/awesome-neovim#tree-sitter-supported-colorscheme
Plug 'sainnhe/sonokai'
" Plug 'liuchengxu/space-vim-theme'

Plug 'darfink/vim-plist'
Plug 'ludovicchabant/vim-gutentags'
Plug 'majutsushi/tagbar'

Plug 'junegunn/fzf', { 'dir': '~/.fzf', 'do': './install --all' }
Plug 'junegunn/fzf.vim'

Plug 'tpope/vim-fugitive'
Plug 'junegunn/gv.vim'

" Plug 'bling/vim-airline'
Plug 'itchyny/lightline.vim'


Plug 'nvim-treesitter/nvim-treesitter', {'do': ':TSUpdate'}

Plug 'sheerun/vim-polyglot'

Plug 'liuchengxu/vim-which-key'

Plug 'Asheq/close-buffers.vim'
Plug 'stefandtw/quickfix-reflector.vim'

Plug 'neomake/neomake'
Plug 'sbdchd/neoformat'

" COMPLETION
Plug 'neovim/nvim-lspconfig'
Plug 'hrsh7th/nvim-compe'

Plug 'hashivim/vim-terraform', {'for': 'terraform'}

Plug 'wellle/context.vim'

Plug 'farfanoide/inflector.vim'

Plug 'rrethy/vim-hexokinase', { 'do': 'make hexokinase' }
let g:Hexokinase_ftEnabled = ['less']
let g:Hexokinase_highlighters = ['backgroundfull']


call plug#end()
