call plug#begin('~/.local/share/nvim/plugged')

" BIG FILES LunarVim/bigfile.nvim

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
" Plug 'sheerun/vim-polyglot'

Plug 'liuchengxu/vim-which-key'

Plug 'Asheq/close-buffers.vim'
Plug 'stefandtw/quickfix-reflector.vim'

Plug 'mfussenegger/nvim-lint'
Plug 'sbdchd/neoformat'

" Plug 'L3MON4D3/LuaSnip', {'tag': 'v2.*', 'do': 'make install_jsregexp'}
" https://ejmastnak.com/tutorials/vim-latex/luasnip/

" COMPLETION
Plug 'neovim/nvim-lspconfig'
" Plug 'hrsh7th/nvim-compe' -- DROP ONCE nvim-cmp is set up
Plug 'hrsh7th/cmp-nvim-lsp'
Plug 'hrsh7th/cmp-buffer'
Plug 'hrsh7th/cmp-path'
Plug 'hrsh7th/cmp-cmdline'
Plug 'hrsh7th/nvim-cmp'

Plug 'j-hui/fidget.nvim'

Plug 'hashivim/vim-terraform', {'for': 'terraform'}

" Plug 'wellle/context.vim'

Plug 'farfanoide/inflector.vim'

Plug 'rrethy/vim-hexokinase', { 'do': 'make hexokinase' }


Plug 'nvim-lua/plenary.nvim'
Plug 'olimorris/codecompanion.nvim'

let g:Hexokinase_ftEnabled = ['less']
let g:Hexokinase_highlighters = ['backgroundfull']


call plug#end()
