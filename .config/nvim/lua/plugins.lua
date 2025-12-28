-- Bootstrap lazy.nvim
local lazypath = vim.fn.stdpath("data") .. "/lazy/lazy.nvim"
if not vim.loop.fs_stat(lazypath) then
  vim.fn.system({
    "git",
    "clone",
    "--filter=blob:none",
    "https://github.com/folke/lazy.nvim.git",
    "--branch=stable",
    lazypath,
  })
end
vim.opt.rtp:prepend(lazypath)

require("lazy").setup({
  -- THEMES
  { "sainnhe/sonokai" },

  -- Filetypes
  { "darfink/vim-plist" },

  -- Tags
  { "ludovicchabant/vim-gutentags" },
  { "majutsushi/tagbar" },

  -- Fuzzy finder
  { "junegunn/fzf", dir = "~/.fzf", build = "./install --all" },
  { "junegunn/fzf.vim" },

  -- Git
  { "tpope/vim-fugitive" },
  { "junegunn/gv.vim" },

  -- Statusline
  { "itchyny/lightline.vim" },

  -- Treesitter
  {
    "nvim-treesitter/nvim-treesitter",
    build = ":TSUpdate",
    config = function()
      require("nvim-treesitter").setup({
        ensure_installed = { "vim", "vimdoc", "rust", "python", "lua" },
      })
      -- Enable treesitter highlighting
      vim.api.nvim_create_autocmd("FileType", {
        callback = function()
          pcall(vim.treesitter.start)
        end,
      })
    end,
  },

  -- Keybinding help
  { "liuchengxu/vim-which-key" },

  -- Buffer management
  { "Asheq/close-buffers.vim" },
  { "stefandtw/quickfix-reflector.vim" },

  -- Linting and formatting
  { "mfussenegger/nvim-lint" },
  { "sbdchd/neoformat" },

  -- LSP and completion
  { "neovim/nvim-lspconfig" },
  { "hrsh7th/cmp-nvim-lsp" },
  { "hrsh7th/cmp-buffer" },
  { "hrsh7th/cmp-path" },
  { "hrsh7th/cmp-cmdline" },
  { "hrsh7th/nvim-cmp" },

  -- LSP progress
  { "j-hui/fidget.nvim" },

  -- Terraform
  { "hashivim/vim-terraform", ft = "terraform" },

  -- Text manipulation
  { "farfanoide/inflector.vim" },

  -- Color preview
  {
    "rrethy/vim-hexokinase",
    build = "make hexokinase",
    init = function()
      vim.g.Hexokinase_ftEnabled = { "less" }
      vim.g.Hexokinase_highlighters = { "backgroundfull" }
    end,
  },

  -- Utilities
  { "nvim-lua/plenary.nvim" },

  -- AI
  { "olimorris/codecompanion.nvim" },
})
