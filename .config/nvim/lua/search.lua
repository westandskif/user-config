local M = {}

-- Build quickfix list from selected files
local function build_quickfix_list(lines)
  local qf_list = vim.tbl_map(function(line)
    return { filename = line }
  end, lines)
  vim.fn.setqflist(qf_list)
  vim.cmd('copen')
end

function M.setup()
  -- FZF actions
  vim.g.fzf_action = {
    ['ctrl-q'] = build_quickfix_list,
    ['ctrl-t'] = 'tab split',
    ['ctrl-x'] = 'split',
    ['ctrl-v'] = 'vsplit',
  }

  -- FZF default options
  vim.env.FZF_DEFAULT_OPTS = '--bind ctrl-a:select-all,ctrl-f:preview-page-down,ctrl-b:preview-page-up --ansi'
  vim.env.FZF_DEFAULT_COMMAND = 'fd --type f --hidden --ignore-file=.ignore'

  -- FZF preview window
  vim.g.fzf_preview_window = { 'right,50%', '?' }

  -- Set tags from TAG_DIRS environment variable
  local function set_tags()
    local tag_dirs = vim.fn.systemlist('echo $TAG_DIRS')
    for _, path in ipairs(tag_dirs) do
      local current_tags = vim.o.tags
      if not current_tags:find(path, 1, true) then
        vim.o.tags = current_tags .. ',' .. path .. '/tags'
      end
    end
    return tag_dirs
  end

  -- Call set_tags on VimEnter
  vim.api.nvim_create_autocmd('VimEnter', {
    callback = set_tags,
    once = true,
  })
end

-- Auto-setup when module is loaded
M.setup()

return M
