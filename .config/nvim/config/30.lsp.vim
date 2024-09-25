set completeopt=menuone,noselect

lua <<EOF
  vim.diagnostic.config({
    underline = false,
    virtual_text = true,
    severity_sort = true,
  })
  vim.api.nvim_create_autocmd('DiagnosticChanged', {
    callback = function(args)
      vim.diagnostic.setloclist()
      -- local diagnostics = args.data.diagnostics
      -- vim.print(diagnostics)
    end,
  })
  -- Set up nvim-cmp.
  local cmp = require'cmp'

  cmp.setup({
    completion = {
        autocomplete = false
    },
    window = {
      -- completion = cmp.config.window.bordered(),
      -- documentation = cmp.config.window.bordered(),
    },
    mapping = cmp.mapping.preset.insert({
      -- ['<C-b>'] = cmp.mapping.scroll_docs(-4),
      -- ['<C-f>'] = cmp.mapping.scroll_docs(4),
      -- ['<C-Space>'] = cmp.mapping.complete(),
      ['<esc>'] = cmp.mapping(function()
        if cmp.visible() then
          local key = vim.api.nvim_replace_termcodes("<c-e>", true, false, true)
          vim.api.nvim_feedkeys(key, 'n', false)
          cmp.abort()
        else
          cmp.complete()
        end
      end),
      ['<Tab>'] = cmp.mapping(function()
        if cmp.visible() then
          cmp.select_next_item({behavior = 'insert'})
        else
          local key = vim.api.nvim_replace_termcodes("<c-n>", true, false, true)
          vim.api.nvim_feedkeys(key, 'n', false)
                
          -- vim.cmd("normal gv=gv")
        end
      end),
      -- ['<C-e>'] = cmp.mapping.abort(),
      ['<CR>'] = cmp.mapping.confirm({ select = true }), -- Accept currently selected item. Set `select` to `false` to only confirm explicitly selected items.
    }),
    sources = cmp.config.sources({
      { name = 'nvim_lsp' },
      -- { name = 'vsnip' }, -- For vsnip users.
      -- { name = 'luasnip' }, -- For luasnip users.
      -- { name = 'ultisnips' }, -- For ultisnips users.
      -- { name = 'snippy' }, -- For snippy users.
    }, {
    {
      name = 'buffer',
      -- option = {
      --   get_bufnrs = function()
      --     return vim.api.nvim_list_bufs()
      --   end
      -- }
    }})
  })

  -- Use buffer source for `/` and `?` (if you enabled `native_menu`, this won't work anymore).
  cmp.setup.cmdline({ '/', '?' }, {
    mapping = cmp.mapping.preset.cmdline(),
    sources = {
      { name = 'buffer' }
    }
  })

  -- Use cmdline & path source for ':' (if you enabled `native_menu`, this won't work anymore).
  cmp.setup.cmdline(':', {
    mapping = cmp.mapping.preset.cmdline(),
    sources = cmp.config.sources({
      { name = 'path' }
    }, {
      { name = 'cmdline' }
    }),
    matching = { disallow_symbol_nonprefix_matching = false }
  })

  -- Set up lspconfig.
  local capabilities = require('cmp_nvim_lsp').default_capabilities()

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
    buf_set_keymap('n', 'gi', '<cmd>lua vim.lsp.buf.implementation()<CR>', opts)
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
  nvim_lsp.tsserver.setup{
      -- cmd = {"/home/nik/work/aprenita/.githooks/bin/typescript-language-server", "--stdio"},
      cmd = {"typescript-language-server", "--stdio"},
      on_attach = on_attach,
  }

  -- C
  nvim_lsp.ccls.setup{
      on_attach = on_attach,
      capabilities = capabilities,
  }

EOF
