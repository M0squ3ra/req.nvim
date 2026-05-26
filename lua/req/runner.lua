local M = {}
local output = require("req.output")
local selection = require("req.selection")

local last_request = nil

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

local function is_inside(path, root)
  path = vim.fs.normalize(path)
  root = vim.fs.normalize(root)

  return path == root or vim.startswith(path, root .. "/")
end

local function current_buffer_dir()
  local name = vim.api.nvim_buf_get_name(0)

  if name == "" then
    return vim.fn.getcwd()
  end

  return vim.fs.dirname(name)
end

local function env_path()
  local cwd = vim.fs.normalize(vim.fn.getcwd())
  local dir = vim.fs.normalize(current_buffer_dir())

  if not is_inside(dir, cwd) then
    dir = cwd
  end

  while true do
    local path = vim.fs.joinpath(dir, ".req", "env.json")

    if vim.fn.filereadable(path) == 1 then
      return path
    end

    if dir == cwd then
      return nil
    end

    local parent = vim.fs.dirname(dir)

    if parent == dir or not is_inside(parent, cwd) then
      return nil
    end

    dir = parent
  end
end

local function context()
  local path = env_path()

  if not path then
    return nil
  end

  local lines = vim.fn.readfile(path)
  return {
    json = table.concat(lines, "\n"),
    path = path,
  }
end

local function command(bin, extra_args)
  local cmd = { bin }
  for _, arg in ipairs(extra_args or {}) do
    table.insert(cmd, arg)
  end

  local ctx = context()

  if ctx then
    table.insert(cmd, "--context-json")
    table.insert(cmd, ctx.json)
  end

  return cmd, ctx and ctx.path or nil
end

local function error_message(stderr, context_path)
  if context_path and stderr and string.find(stderr, "Invalid context JSON", 1, true) then
    return stderr .. "\nContext file: " .. context_path
  end

  return stderr
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

local function run_input(bin, input, extra_args, filetype, save_last)
  local cmd, context_path = command(bin, extra_args)

  if save_last then
    last_request = {
      input = input,
      extra_args = extra_args,
      filetype = filetype,
    }
  end

  vim.system(cmd, { text = true, stdin = input }, function(result)
    vim.schedule(function()
      if result.code ~= 0 then
        vim.notify(error_message(result.stderr, context_path), vim.log.levels.ERROR)
        return
      end

      if result.stderr and result.stderr ~= "" then
        vim.notify(error_message(result.stderr, context_path), vim.log.levels.WARN)
      end

      output.show(result.stdout, filetype)
    end)
  end)
end

local function execute(opts, extra_args, filetype, save_last)
  local bin = executable_bin()

  if not bin then
    vim.notify(
      "req.nvim: Rust binary not found. Run `cargo build` or `cargo build --release`.",
      vim.log.levels.ERROR
    )
    return
  end

  request_input(bin, opts, function(input)
    run_input(bin, input, extra_args, filetype, save_last)
  end)
end

function M.run(opts)
  execute(opts, {}, "req_response", true)
end

function M.curl(opts)
  execute(opts, { "--export-curl" }, "sh", false)
end

function M.rerun()
  if not last_request then
    vim.notify("req.nvim: no request to rerun", vim.log.levels.ERROR)
    return
  end

  local bin = executable_bin()

  if not bin then
    vim.notify(
      "req.nvim: Rust binary not found. Run `cargo build` or `cargo build --release`.",
      vim.log.levels.ERROR
    )
    return
  end

  run_input(bin, last_request.input, last_request.extra_args, last_request.filetype, true)
end

return M
