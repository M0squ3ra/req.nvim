local M = {}
local output = require("req.output")
local selection = require("req.selection")

local function plugin_root()
  local source = debug.getinfo(1, "S").source:sub(2)
  return vim.fs.dirname(vim.fs.dirname(vim.fs.dirname(source)))
end

local function executable_bin()
  local root = plugin_root()
  local release = vim.fs.joinpath(root, "target", "release", "req-nvim")
  local debug = vim.fs.joinpath(root, "target", "debug", "req-nvim")

  if vim.fn.executable(release) == 1 then
    return release
  end

  if vim.fn.executable(debug) == 1 then
    return debug
  end
end

local function context_json()
  local path = vim.fs.joinpath(vim.fn.getcwd(), ".req", "env.json")

  if vim.fn.filereadable(path) ~= 1 then
    return nil
  end

  local lines = vim.fn.readfile(path)
  return table.concat(lines, "\n")
end

local function command(bin, extra_args)
  local cmd = { bin }
  for _, arg in ipairs(extra_args or {}) do
    table.insert(cmd, arg)
  end

  local context = context_json()

  if context then
    table.insert(cmd, "--context-json")
    table.insert(cmd, context)
  end

  return cmd
end

local function request_input(bin, opts, callback)
  if opts and opts.range > 0 then
    callback(selection.visual())
    return
  end

  local buffer = selection.buffer()

  vim.system({ bin, "--list-requests" }, { text = true, stdin = buffer }, function(result)
    vim.schedule(function()
      if result.code ~= 0 then
        vim.notify(result.stderr, vim.log.levels.ERROR)
        return
      end

      local ok, requests = pcall(vim.json.decode, result.stdout)
      if not ok then
        vim.notify("req.nvim: failed to parse request list", vim.log.levels.ERROR)
        return
      end

      local request = selection.find_at_cursor(requests)
      if not request then
        vim.notify("req.nvim: no request under cursor", vim.log.levels.ERROR)
        return
      end

      callback(selection.range(request.start_line, request.end_line))
    end)
  end)
end

local function execute(opts, extra_args, filetype)
  local bin = executable_bin()

  if not bin then
    vim.notify(
      "req.nvim: Rust binary not found. Run `cargo build` or `cargo build --release`.",
      vim.log.levels.ERROR
    )
    return
  end

  request_input(bin, opts, function(input)
    vim.system(command(bin, extra_args), { text = true, stdin = input }, function(result)
      vim.schedule(function()
        if result.code ~= 0 then
          vim.notify(result.stderr, vim.log.levels.ERROR)
          return
        end

        output.show(result.stdout, filetype)
      end)
    end)
  end)
end

function M.run(opts)
  execute(opts, {}, "req_response")
end

function M.curl(opts)
  execute(opts, { "--export-curl" }, "sh")
end

return M
