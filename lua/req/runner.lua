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

function M.run(opts)
  local bin = executable_bin()

  if not bin then
    vim.notify(
      "req.nvim: Rust binary not found. Run `cargo build` or `cargo build --release`.",
      vim.log.levels.ERROR
    )
    return
  end

  local cmd = { bin }
  local context = context_json()

  if context then
    table.insert(cmd, "--context-json")
    table.insert(cmd, context)
  end

  vim.system(cmd, { text = true, stdin = selection.input(opts) }, function(result)
    vim.schedule(function()
      if result.code ~= 0 then
        vim.notify(result.stderr, vim.log.levels.ERROR)
        return
      end

      output.show(result.stdout)
    end)
  end)
end

return M
