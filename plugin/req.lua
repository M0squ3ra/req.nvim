local subcommands = {
	run = function(opts)
		require("req").run(opts)
	end,
	curl = function(opts)
		require("req").curl(opts)
	end,
	rerun = function()
		require("req").rerun()
	end,
}

vim.api.nvim_create_user_command("Req", function(opts)
	local command = opts.fargs[1] or "run"
	local handler = subcommands[command]

	if not handler then
		vim.notify("req.nvim: unknown command `" .. command .. "`", vim.log.levels.ERROR)
		return
	end

	handler(opts)
end, {
	nargs = "?",
	range = true,
	complete = function()
		return vim.tbl_keys(subcommands)
	end,
})
