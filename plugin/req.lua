vim.api.nvim_create_user_command("ReqHello", function()
  require("req").hello()
end, {})
