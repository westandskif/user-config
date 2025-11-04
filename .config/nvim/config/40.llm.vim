lua <<EOF
require("codecompanion").setup({
  adapters = { http = {
    qwen_lmstudio = function()
      return require("codecompanion.adapters").extend("openai_compatible", {
        name = "lmstudio",
        env = {
          url = "http://127.0.0.1:1234",  -- LM Studio OpenAI-compatible base URL
          api_key = "lm-studio",             -- LM Studio accepts any string; this avoids empty-key warnings
          chat_url = "/v1/chat/completions",
          models_endpoint = "/v1/models",
        },
        schema = {
          -- IMPORTANT: set to the exact model id shown in LM Studio's server UI
          -- model = { default = "qwen/qwen3-coder-30b" },
          model = { default = "qwen3-coder-30b-a3b-instruct-mlx@6bit" },
          temperature = { default = 0.0 },
        },
      })
    end,
    xai = function()
      return require("codecompanion.adapters").extend("xai", {
        env = {
          api_key = "xai-secret-goes-here",
        },
        schema = {
          model = {
            default = "grok-code-fast-1",
          },
        },
      })
    end,
    opts = {
      allow_insecure = true,
      proxy = "socks5://localhost:1080",
    },
  },
  },
  strategies = {
    chat   = {
        adapter = "qwen_lmstudio"
    },
    inline = { adapter = "qwen_lmstudio" },
    cmd    = { adapter = "qwen_lmstudio" },
  },
  -- strategies = {
  --   chat   = {
  --       adapter = "xai"
  --   },
  --   inline = { adapter = "xai" },
  --   cmd    = { adapter = "xai" },
  -- },
})
EOF
