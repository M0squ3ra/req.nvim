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

function M.run(opts)
  local bin = executable_bin()

  if not bin then
    vim.notify(
      "req.nvim: Rust binary not found. Run `cargo build` or `cargo build --release`.",
      vim.log.levels.ERROR
    )
    return
  end

  vim.system({ bin }, { text = true, stdin = selection.input(opts) }, function(result)
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
