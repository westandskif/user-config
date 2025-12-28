-- Tweak highlights
local function tweak_highlights()
  vim.api.nvim_set_hl(0, "SpellBad", { underline = true })
end

vim.api.nvim_create_autocmd("ColorScheme", {
  pattern = "*",
  callback = tweak_highlights,
})

vim.cmd.colorscheme("sonokai")

-- Protect changes between writes
vim.opt.swapfile = true
vim.opt.updatetime = 10000
-- Protect against crash-during-write
vim.opt.writebackup = true
-- Do not persist backup after successful write
vim.opt.backup = false
-- Use rename-and-write-new method whenever safe
vim.opt.backupcopy = "auto"
-- Persist the undo tree for each file
vim.opt.undofile = true

vim.opt.termguicolors = true
vim.opt.hlsearch = true
vim.opt.incsearch = true
vim.opt.number = true
vim.opt.wrap = false
vim.opt.smartcase = true
vim.opt.ignorecase = true
vim.opt.splitright = true
vim.opt.splitbelow = true

vim.opt.signcolumn = "yes:1"
vim.opt.encoding = "utf-8"
vim.opt.laststatus = 2
vim.cmd("syntax on")
vim.opt.pumheight = 20
vim.opt.redrawtime = 2000
vim.opt.clipboard = "unnamedplus"

-- Netrw settings
vim.g.netrw_altv = 1
vim.g.netrw_alto = 1

-- Tagbar settings
vim.g.tagbar_foldlevel = 0
vim.g.tagbar_autoclose = 1
vim.g.tagbar_sort = 0

-- Context settings
vim.g.context_enabled = 0
vim.g.context_presenter = "preview"

-- Quickfix settings
vim.g.qf_modifiable = 1
vim.g.qf_join_changes = 1
vim.g.qf_write_changes = 0
