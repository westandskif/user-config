local M = {}

local function path_as_import(path, name)
  path = path:gsub("/", ".")
  path = path:gsub("%.py$", "")
  return "from " .. path .. " import " .. name
end

local function do_import(import_path)
  vim.cmd("normal! O" .. import_path)
end

function M.auto_import(name)
  -- Bypass tagfunc, use tags file directly
  local saved_tagfunc = vim.bo.tagfunc
  vim.bo.tagfunc = ""
  
  local tags = vim.fn.taglist("^" .. name .. "$")
  
  vim.bo.tagfunc = saved_tagfunc

  -- Filter top-level tags
  local filtered_tags = vim.tbl_filter(function(tag)
    local char = tag.cmd:sub(3, 3)
    return char ~= " "
  end, tags)

  local matched_files = vim.tbl_map(function(tag)
    return tag.filename
  end, filtered_tags)

  -- Strip base path from each tagfile
  local tagfiles = vim.fn.tagfiles()

  for _, tagfile in ipairs(tagfiles) do
    local base = vim.fn.fnamemodify(tagfile, ":p:h")
    matched_files = vim.tbl_map(function(file)
      return (file:gsub("^" .. vim.pesc(base) .. "/", ""))
    end, matched_files)
  end

  -- Prepare import strings
  local imports = vim.tbl_map(function(file)
    return path_as_import(file, name)
  end, matched_files)

  local size = #imports
  if size < 1 then
    vim.api.nvim_echo({ { "[AutoImport] Not found!", "WarningMsg" } }, false, {})
    return
  end

  if size == 1 then
    do_import(imports[1])
    return
  end

  vim.fn["fzf#run"](vim.fn["fzf#wrap"]({
    source = imports,
    sink = do_import,
  }))
end

local function on_lint_finish(callback)
    local poll = require("fidget.poll")
    local lint = require("lint")

    local poller = poll.Poller {
        name = "Linting",
        poll = function()
            local linters = lint.get_running()
            if #linters > 0 then
                return true
            else
                pcall(callback)
                return false
            end
        end
    }

    poller:start_polling(25)
end
local function try_lint(prg)
    local callback = function()
        vim.diagnostic.setloclist()
    end
    require("lint").try_lint(prg)
    on_lint_finish(callback)
end

-- Python-specific settings and keymaps
function M.setup()
    local python_group = vim.api.nvim_create_augroup('PythonSpecifics', { clear = true })

    vim.api.nvim_create_autocmd('FileType', {
        group = python_group,
        pattern = 'python',
        callback = function()
            local opts = { buffer = true, silent = true }

            -- Quick lint with ruff
            vim.keymap.set('n', '<localleader>lmq', function()
                vim.diagnostic.reset()
                try_lint('ruff')
            end, opts)

            -- Lint with mypy
            vim.keymap.set('n', '<localleader>lma', function()
                vim.diagnostic.reset()
                try_lint('mypy')
            end, opts)

            -- Auto import
            vim.keymap.set('n', '<localleader>lai', function()
                M.auto_import(vim.fn.expand('<cword>'))
            end, opts)
        end,
    })

    -- Python-specific settings
    vim.g.python_pep8_indent_searchpair_timeout = 20
end

-- Auto-setup when module is loaded
M.setup()

return M
