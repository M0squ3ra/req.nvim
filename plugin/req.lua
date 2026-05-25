vim.api.nvim_create_user_command("ReqRun", function(opts)
	require("req").run(opts)
end, { range = true })

vim.api.nvim_create_user_command("ReqCurl", function(opts)
	require("req").curl(opts)
end, { range = true })
