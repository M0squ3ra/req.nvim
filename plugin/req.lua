vim.api.nvim_create_user_command("ReqRun", function(opts)
	require("req").run(opts)
end, { range = true })
