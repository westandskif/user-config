-- Put this in your init.lua (or in lua/keymaps.lua and `require("keymaps")`)

-- ========== LEADER ==========
vim.g.mapleader = " "
vim.g.maplocalleader = " "

-- small helpers
local map = vim.keymap.set
local function t(keys) return vim.api.nvim_replace_termcodes(keys, true, false, true) end
local function feed(keys, mode) vim.api.nvim_feedkeys(t(keys), mode or "n", false) end

-- ========== GENERAL ==========
map("n", "<Esc>", function()
  -- equivalent to: :let @/=""
  vim.fn.setreg("/", "")
end, { silent = true, noremap = true })

for _, lhs in ipairs({ "jk", "Jk", "jK", "JK" }) do
  map("i", lhs, "<Esc>", { noremap = true })
end

map("t", "jk", [[<C-\><C-n>]], { noremap = true })
map("i", "0P", "<C-r>0", { noremap = true })

map("n", "gd", function() vim.lsp.buf.definition() end, { silent = true, noremap = true })

map("n", "<Up>", "Nzz", { noremap = true })
map("n", "<Down>", "nzz", { noremap = true })

-- "inner line" textobject
map({ "o", "x" }, "il", ":<C-u>normal! _vg_<CR>", { silent = true, noremap = true })

-- paste from yank register 0 in visual
map("x", "P", '"0p', { noremap = true })

-- function keys
map("n", "<F1>", "<C-y>", { noremap = true })
map("n", "<F2>", "<C-e>", { noremap = true })
map("n", "<F3>", "<C-u>", { noremap = true })
map("n", "<F4>", "<C-d>", { noremap = true })
map("n", "<F5>", "zH", { noremap = true })
map("n", "<F6>", "zL", { noremap = true })
map("n", "<F7>", ":normal! zszH<CR>", { noremap = true, silent = true })

map("x", "<F1>", "<C-y>", { noremap = true })
map("x", "<F2>", "<C-e>", { noremap = true })
map("x", "<F3>", "<C-u>", { noremap = true })
map("x", "<F4>", "<C-d>", { noremap = true })
map("x", "<F5>", "zH", { noremap = true })
map("x", "<F6>", "zL", { noremap = true })

-- commands: W/Q/Wq
vim.api.nvim_create_user_command("W", function() vim.cmd("w") end, {})
vim.api.nvim_create_user_command("Q", function() vim.cmd("q") end, {})
vim.api.nvim_create_user_command("Wq", function() vim.cmd("wq") end, {})

-- move between windows
map("n", "gh", "<C-w>h", { noremap = true })
map("n", "gl", "<C-w>l", { noremap = true })
map("n", "gj", "<C-w>j", { noremap = true })
map("n", "gk", "<C-w>k", { noremap = true })

map("n", "gH", "mW<C-w>h`Wzz", { noremap = true })
map("n", "gL", "mW<C-w>l`Wzz", { noremap = true })
map("n", "gJ", "mW<C-w>j`Wzz", { noremap = true })
map("n", "gK", "mW<C-w>k`Wzz", { noremap = true })

-- netrw mappings (buffer-local)
local netrw_grp = vim.api.nvim_create_augroup("netrw_mapping", { clear = true })
vim.api.nvim_create_autocmd("FileType", {
  group = netrw_grp,
  pattern = "netrw",
  callback = function(args)
    local b = args.buf
    map("n", "gh", "<C-w>h", { buffer = b, noremap = true })
    map("n", "gl", "<C-w>l", { buffer = b, noremap = true })
    map("n", "gj", "<C-w>j", { buffer = b, noremap = true })
    map("n", "gk", "<C-w>k", { buffer = b, noremap = true })
    -- <Plug> mappings must be remappable
    map("n", "-", "<Plug>NetrwBrowseUpDir", { buffer = b, remap = true })
  end,
})

-- ========== WHICH-KEY (vim-which-key plugin style) ==========
-- NOTE: This matches your Vimscript style (liuchengxu/vim-which-key).
-- If you're actually using folke/which-key.nvim, this section would be different.
vim.g.which_key_map = {}

vim.g.which_key_map.v = {
  name = "☰ NVIM",
  e = "Edit configs",
  s = "Source configs",
  q = "Quit",
  C = "Cache session",
  u = "Quit and make session",
  R = "Restore session",
}

map("n", "<leader>ve", "<cmd>tabnew ~/.config/nvim/config<CR>", { noremap = true, silent = true })
map("n", "<leader>vs", "<cmd>source ~/.config/nvim/init.vim<CR>", { noremap = true, silent = true })
map("n", "<leader>vc", "<cmd>mksession! .s.vim<CR>", { noremap = true, silent = true })
map("n", "<leader>vq", "<cmd>mksession! .s.vim<CR><cmd>qa<CR>", { noremap = true, silent = true })
map("n", "<leader>vr", "<cmd>source .s.vim<CR>", { noremap = true, silent = true })

vim.g.which_key_map.m = { name = "☰ MODE", t = "toggle tabbar", c = "toggle context" }
map("n", "<leader>mt", "<cmd>TagbarToggle<CR>", { noremap = true, silent = true })
map("n", "<leader>mc", "<cmd>ContextToggle<CR>", { noremap = true, silent = true })

-- ========== QUICKFIX / LOCLIST helpers ==========
local function is_loclist_open()
  for _, win in ipairs(vim.fn.getwininfo()) do
    if win.quickfix == 1 and win.loclist == 1 then
      return true
    end
  end
  return false
end

local function populate_qf(cmd)
  vim.fn.setqflist({}, " ", { lines = vim.fn.systemlist(cmd), efm = "%f" })
  vim.cmd("copen")
end

vim.api.nvim_create_user_command("CommandToQf", function(opts)
  populate_qf(opts.args)
end, { nargs = 1 })

vim.g.which_key_map.q = {
  name = "☰ QUICKFIX / LOCLIST",
  C = "copy loclist to quickfix",
  q = "quit",
  r = "reset diagnostic & drop loclist",
  d = "diagnostic to loclist",
}

map("n", "<leader>qr", function()
  vim.diagnostic.reset()
  vim.fn.setqflist({})
end, { noremap = true, silent = true })

map("n", "<leader>qd", function()
  vim.diagnostic.setloclist()
end, { noremap = true, silent = true })

map("n", "<leader>qC", function()
  vim.fn.setqflist(vim.fn.getloclist(0))
  vim.cmd("lclose")
  vim.cmd("copen")
end, { noremap = true, silent = true })

map("n", "<leader>qq", function()
  if is_loclist_open() then
    vim.cmd("lclose")
  else
    vim.cmd("cclose")
  end
end, { noremap = true, silent = true })

map("n", "gC", function()
  if is_loclist_open() then
    vim.cmd("ll")
  else
    vim.cmd("cc")
  end
  vim.cmd("normal! zz")
end, { noremap = true, silent = true })

map("n", "gn", function()
  if is_loclist_open() then
    vim.cmd("lnext")
  else
    vim.cmd("cnext")
  end
  vim.cmd("normal! zz")
end, { noremap = true, silent = true })

map("n", "gN", function()
  if is_loclist_open() then
    vim.cmd("lnfile")
  else
    vim.cmd("cnfile")
  end
  vim.cmd("normal! zz")
end, { noremap = true, silent = true })

map("n", "gp", function()
  if is_loclist_open() then
    vim.cmd("lprevious")
  else
    vim.cmd("cprevious")
  end
  vim.cmd("normal! zz")
end, { noremap = true, silent = true })

map("n", "gP", function()
  if is_loclist_open() then
    vim.cmd("lpfile")
  else
    vim.cmd("cpfile")
  end
  vim.cmd("normal! zz")
end, { noremap = true, silent = true })

-- ========== TAB ==========
vim.g.which_key_map.t = { name = "☰ TAB", n = "new", m = "move", q = "quit" }
map("n", "<leader>tn", "<cmd>tabnew<CR>", { noremap = true, silent = true })
map("n", "<leader>tm", ":tabmove ", { noremap = true, silent = true })
map("n", "<leader>tq", "<cmd>tabclose<CR>", { noremap = true, silent = true })

-- ========== JUMP ==========
vim.g.which_key_map.j = { name = "☰ JUMP", c = "Current buffer dir", n = "Notes", r = "Root dir" }
map("n", "<leader>jc", "<cmd>e %:p:h<CR>", { noremap = true, silent = true })
map("n", "<leader>jn", "<cmd>e .notes<CR>", { noremap = true, silent = true })
map("n", "<leader>jr", "<cmd>e .<CR>", { noremap = true, silent = true })

-- ========== SEARCH helpers ==========
local function input_str(prompt)
  vim.fn.inputsave()
  local s = vim.fn.input(prompt)
  vim.fn.inputrestore()
  vim.fn.setreg("i", s)
  return s
end

local function vim_escape(str, symbols_to_escape)
  local symbols = symbols_to_escape or ""
  local out = str or ""
  -- mimic your Vimscript special-case for single quote
  if symbols:find("'", 1, true) then
    symbols = symbols:gsub("'", "")
    out = out:gsub("'", "''")
  end
  return vim.fn.escape(out, symbols)
end

local function hist_add_n_run(cmd)
  vim.fn.histadd("cmd", cmd)
  vim.cmd(cmd)
end

local function visual_to_reg_i()
  local mode_now = vim.api.nvim_get_mode().mode
  if mode_now ~= 'v' and mode_now ~= 'V' and mode_now ~= "\22" then
    error("visual_to_reg_i() must be called from visual mode")
  end

  local ok = pcall(vim.cmd, [[silent normal! "iy]])
  if not ok then
    error("Failed to yank visual selection into register i")
  end

  local txt = vim.fn.getreg("i"):gsub("\n+$", "")
  vim.fn.setreg("i", txt)
  return txt
end

-- :Rg command (fzf.vim)
vim.api.nvim_create_user_command("Rg", function(opts)
  local base = "rg --column --line-number --no-heading --hidden --sort=path "
  local fzf_cmd = base .. "--color=always " .. opts.args
  local ok = pcall(function()
    return vim.fn["fzf#vim#grep"](fzf_cmd, 1, vim.fn["fzf#vim#with_preview"](), 0)
  end)
  if not ok then
    populate_qf(base .. opts.args)
  end
end, { nargs = "+", bang = true, complete = "dir" })

vim.g.which_key_map.s = {
  name = "☰ SEARCH",
  P = '"+" globally',
  b = "buffers",
  c = "custom Rg opts",
  e = "exact",
  f = "files",
  h = "history",
  l = "exact Locally",
  p = '"+" locally',
  r = "Rg regex with PCRE2",
  t = "tags",
  w = "exact words",
}

vim.g.sym_to_escape_for_rg = [[$#%\"`]]
vim.g.sym_to_escape_for_rg_regex = [[#%"']]
vim.g.sym_to_escape_for_buffer_search = "/\\"

map("n", "<leader>sb", "<cmd>Buffers<CR>", { noremap = true, silent = true })
map("n", "<leader>sf", "<cmd>FZF<CR>", { noremap = true, silent = true })
map("n", "<leader>sh", "<cmd>History<CR>", { noremap = true, silent = true })
map("n", "<leader>st", "<cmd>Tags<CR>", { noremap = true, silent = true })

-- exact search locally (opens "/" prompt, no <CR> just like your original)
map("n", "<leader>sl", function()
  local s = input_str("exact: ")
  local esc = vim_escape(s, vim.g.sym_to_escape_for_buffer_search)
  vim.fn.setreg("i", esc)
  feed("/\\C\\V" .. esc, "n")
end, { noremap = true })

map("x", "<leader>sl", function()
  local s = visual_to_reg_i()
  local esc = vim_escape(s, vim.g.sym_to_escape_for_buffer_search)
  vim.fn.setreg("i", esc)
  feed("<Esc>/\\C\\V" .. esc, "n")
end, { noremap = true })

-- custom opts (opens ":" prompt with command prefilled, no <CR>)
map("n", "<leader>sc", function()
  local s = input_str("search: ")
  local esc = vim_escape(s, vim.g.sym_to_escape_for_rg)
  vim.fn.setreg("i", esc)

  local cmd = ('Rg --no-ignore-vcs -S -- "%s"'):format(esc)
  vim.fn.histadd("cmd", cmd)
  feed(":" .. cmd, "n")
end, { noremap = true })

-- regex (opens ":" prompt prefilled, no <CR>)
map("n", "<leader>sr", function()
  local s = input_str("regex: ")
  local esc = vim_escape(s, vim.g.sym_to_escape_for_rg_regex)
  vim.fn.setreg("i", esc)

  local cmd = ('Rg --no-ignore-vcs --pcre2 -- "%s"'):format(esc)
  vim.fn.histadd("cmd", cmd)
  feed(":" .. cmd, "n")
end, { noremap = true })

map("x", "<leader>sr", function()
  local s = visual_to_reg_i()
  local esc = vim_escape(s, vim.g.sym_to_escape_for_rg_regex)
  vim.fn.setreg("i", esc)

  local cmd = ('Rg --no-ignore-vcs --pcre2 -- "%s"'):format(esc)
  vim.fn.histadd("cmd", cmd)
  feed("<Esc>:" .. cmd, "n")
end, { noremap = true })

-- exact (runs immediately)
map("n", "<leader>se", function()
  local s = input_str(" exact: ")
  local esc = vim_escape(s, vim.g.sym_to_escape_for_rg)
  vim.fn.setreg("i", esc)

  local cmd = ('Rg --no-ignore-vcs -F -- "%s"'):format(esc)
  hist_add_n_run(cmd)
end, { noremap = true, silent = true })

map("x", "<leader>se", function()
  local s = visual_to_reg_i()
  local esc = vim_escape(s, vim.g.sym_to_escape_for_rg)
  vim.fn.setreg("i", esc)

  local cmd = ('Rg --no-ignore-vcs -F -- "%s"'):format(esc)
  hist_add_n_run(cmd)
end, { noremap = true, silent = true })

-- exact words (runs immediately)
map("n", "<leader>sw", function()
  local s = input_str(" words: ")
  local esc = vim_escape(s, vim.g.sym_to_escape_for_rg)
  vim.fn.setreg("i", esc)

  local cmd = ('Rg --no-ignore-vcs -F -w -- "%s"'):format(esc)
  hist_add_n_run(cmd)
end, { noremap = true, silent = true })

map("x", "<leader>sw", function()
  local s = visual_to_reg_i()
  local esc = vim_escape(s, vim.g.sym_to_escape_for_rg)
  vim.fn.setreg("i", esc)

  local cmd = ('Rg --no-ignore-vcs -F -w -- "%s"'):format(esc)
  hist_add_n_run(cmd)
end, { noremap = true, silent = true })

-- ========== LANGUAGE ==========
local language_map = { name = "☰ LANGUAGE" }
language_map.m = { name = "☰ Make", q = "Quick", a = "All" }
language_map.m.Q = { name = "☰ Quickfix", Q = "Quick", A = "All" }

language_map.a = { name = "☰ Add", i = "import", w = "word" }
map("n", "<leader>law", "<cmd>SpellingAddWord<CR>", { noremap = true, silent = true })

language_map.f = { name = "☰ Fix", f = "format" }
map("n", "<leader>lff", function() require("conform").format() end, { noremap = true, silent = true })

map("n", "<leader>lfi", function()
    require("conform").format({ formatters = { "isort" } })
end, { noremap = true, silent = true })

vim.g.which_key_map.l = language_map

-- ========== BUFFERS ==========
vim.g.which_key_map.b = { name = "☰ BUFFERS", q = "quit all" }
map("n", "<leader>bq", "<cmd>Bdelete hidden<CR>", { noremap = true, silent = true })

-- ========== COMMANDS ==========
vim.g.which_key_map.c = {
  name = "☰ COMMANDS",
  f = "copy file",
  p = "copy file path",
  t = "Trim whitespaces",
  R = "Refresh buffers & regenerate local tags",
  T = "refresh outer Tags",
}

map("n", "<leader>cp", function()
  local p = vim.fn.expand("%")
  vim.fn.setreg("0", p)
  vim.fn.setreg("+", p)
end, { noremap = true, silent = true })

map("n", "<leader>cP", function()
  local p = vim.fn.expand("%") .. ":" .. tostring(vim.fn.line("."))
  vim.fn.setreg("0", p)
  vim.fn.setreg("+", p)
end, { noremap = true, silent = true })

map("n", "<leader>ct", "<cmd>%s/\\s\\+$//g<CR>", { noremap = true, silent = true })

-- which-key registration + leader popup mappings
pcall(function()
  vim.fn["which_key#register"]("<Space>", "g:which_key_map")
end)

map("n", "<leader>", "<cmd>WhichKey '<Space>'<CR>", { noremap = true, silent = true })
map("x", "<leader>", "<cmd>WhichKeyVisual '<Space>'<CR>", { noremap = true, silent = true })
