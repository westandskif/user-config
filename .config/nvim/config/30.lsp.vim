lua << EOF
local nvim_lsp = require('lspconfig')

local on_init = function(client, initialization_result)
  if client.server_capabilities then
    client.server_capabilities.documentFormattingProvider = false
    client.server_capabilities.semanticTokensProvider = false  -- turn off semantic tokens
  end
end

local on_attach = function(client, bufnr)
  local function buf_set_keymap(...) vim.api.nvim_buf_set_keymap(bufnr, ...) end
  local function buf_set_option(...) vim.api.nvim_buf_set_option(bufnr, ...) end

  --Enable completion triggered by <c-x><c-o>
  -- buf_set_option('omnifunc', 'v:lua.vim.lsp.omnifunc')

  -- Mappings.
  local opts = { noremap=true, silent=true }

  -- See `:help vim.lsp.*` for documentation on any of the below functions
  buf_set_keymap('n', 'gd', '<Cmd>lua vim.lsp.buf.definition()<CR>', opts)
  buf_set_keymap('n', 'K', '<Cmd>lua vim.lsp.buf.hover()<CR>', opts)
  -- buf_set_keymap('n', 'gD', '<Cmd>lua vim.lsp.buf.declaration()<CR>', opts)
  -- buf_set_keymap('n', '<space>e', '<cmd>lua vim.lsp.diagnostic.show_line_diagnostics()<CR>', opts)
  -- buf_set_keymap('n', '<space>lmq', '<cmd>lua vim.lsp.diagnostic.set_loclist()<CR>', opts)
  -- buf_set_keymap("n", "<space>f", "<cmd>lua vim.lsp.buf.formatting()<CR>", opts)
  -- buf_set_keymap('n', 'gi', '<cmd>lua vim.lsp.buf.implementation()<CR>', opts)
  buf_set_keymap('n', '<C-k>', '<cmd>lua vim.lsp.buf.signature_help()<CR>', opts)
  -- buf_set_keymap('n', '<space>wa', '<cmd>lua vim.lsp.buf.add_workspace_folder()<CR>', opts)
  -- buf_set_keymap('n', '<space>wr', '<cmd>lua vim.lsp.buf.remove_workspace_folder()<CR>', opts)
  -- buf_set_keymap('n', '<space>wl', '<cmd>lua print(vim.inspect(vim.lsp.buf.list_workspace_folders()))<CR>', opts)
  -- buf_set_keymap('n', '<space>D', '<cmd>lua vim.lsp.buf.type_definition()<CR>', opts)
  -- buf_set_keymap('n', '<space>rn', '<cmd>lua vim.lsp.buf.rename()<CR>', opts)
  -- buf_set_keymap('n', '<space>ca', '<cmd>lua vim.lsp.buf.code_action()<CR>', opts)
  -- buf_set_keymap('n', 'gr', '<cmd>lua vim.lsp.buf.references()<CR>', opts)
  -- buf_set_keymap('n', '[d', '<cmd>lua vim.lsp.diagnostic.goto_prev()<CR>', opts)
  -- buf_set_keymap('n', ']d', '<cmd>lua vim.lsp.diagnostic.goto_next()<CR>', opts)

end

-- PYTHON
-- available settings at https://github.com/python-lsp/python-lsp-server/blob/develop/pylsp/config/schema.json
nvim_lsp.pylsp.setup{
    on_init = on_init,
    on_attach = on_attach,
    settings = {
        pylsp = {
            plugins = {
                pycodestyle = { enabled = false },
                pylint = { enabled = false },
                yapf = { enabled = false },
                pyflakes = { enabled = false },
                mccabe = { enabled = false },
            }
        }
    }
}

-- RUST
local capabilities = vim.lsp.protocol.make_client_capabilities()
capabilities.textDocument.completion.completionItem.snippetSupport = true
nvim_lsp.rust_analyzer.setup{
    on_attach = on_attach,
    capabilities = capabilities,
}


-- TS
-- nvim_lsp.tsserver.setup{
--     -- cmd = {"/home/nik/work/aprenita/.githooks/bin/typescript-language-server", "--stdio"},
--     cmd = {"typescript-language-server", "--stdio"},
--     on_attach = on_attach,
-- }

EOF

" set completeopt=noinsert,menuone,noselect
set completeopt=menuone,noselect

let g:compe = {}
let g:compe.enabled = v:true
let g:compe.autocomplete = v:false
let g:compe.debug = v:false
let g:compe.min_length = 1
let g:compe.preselect = 'enable'
let g:compe.throttle_time = 80
let g:compe.source_timeout = 200
let g:compe.resolve_timeout = 800
let g:compe.incomplete_delay = 400
let g:compe.max_abbr_width = 100
let g:compe.max_kind_width = 100
let g:compe.max_menu_width = 100
let g:compe.documentation = v:true

let g:compe.source = {}
let g:compe.source.path = v:false
let g:compe.source.buffer = v:false
" let g:compe.source.calc = v:true
let g:compe.source.nvim_lsp = v:true
" let g:compe.source.nvim_lua = v:true
" let g:compe.source.vsnip = v:true
" let g:compe.source.ultisnips = v:true
" let g:compe.source.luasnip = v:true
" let g:compe.source.emoji = v:true
