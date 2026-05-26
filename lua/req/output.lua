local M = {}
local config = require("req.config")

local buffer_count = 0

local function find_response_buf()
  local buffer_name = config.options.output.buffer_name

  for _, bufnr in ipairs(vim.api.nvim_list_bufs()) do
    if vim.api.nvim_buf_is_valid(bufnr) and vim.api.nvim_buf_get_name(bufnr) == buffer_name then
      return bufnr
    end
  end
end

local function next_buffer_name()
  if config.options.window.reuse then
    return config.options.output.buffer_name
  end

  buffer_count = buffer_count + 1
  return string.format("%s#%d", config.options.output.buffer_name, buffer_count)
end

local function get_or_create_buf()
  local bufnr = config.options.window.reuse and find_response_buf() or nil

  if bufnr then
    return bufnr
  end

  bufnr = vim.api.nvim_create_buf(false, true)

  vim.api.nvim_buf_set_name(bufnr, next_buffer_name())
  vim.bo[bufnr].buftype = "nofile"
  vim.bo[bufnr].bufhidden = "wipe"
  vim.bo[bufnr].swapfile = false
  vim.bo[bufnr].filetype = config.options.output.filetype.response

  return bufnr
end

local function open_buf(bufnr)
  local window = config.options.window

  if window.reuse then
    for _, winid in ipairs(vim.api.nvim_list_wins()) do
      if vim.api.nvim_win_get_buf(winid) == bufnr then
        vim.api.nvim_set_current_win(winid)
        return
      end
    end
  end

  if window.position == "current" then
    vim.api.nvim_win_set_buf(0, bufnr)
  elseif window.position == "left" then
    vim.cmd("topleft vertical split")
    vim.api.nvim_win_set_buf(0, bufnr)
  elseif window.position == "bottom" then
    vim.cmd("botright split")
    vim.api.nvim_win_set_buf(0, bufnr)
  elseif window.position == "top" then
    vim.cmd("topleft split")
    vim.api.nvim_win_set_buf(0, bufnr)
  else
    vim.cmd("rightbelow vsplit")
    vim.api.nvim_win_set_buf(0, bufnr)
  end

  if window.size then
    if window.position == "bottom" or window.position == "top" then
      vim.api.nvim_win_set_height(0, window.size)
    elseif window.position ~= "current" then
      vim.api.nvim_win_set_width(0, window.size)
    end
  end
end

function M.show(text, filetype)
  local bufnr = get_or_create_buf()

  open_buf(bufnr)

  vim.bo[bufnr].filetype = filetype or config.options.output.filetype.response
  vim.bo[bufnr].modifiable = true

  local lines = vim.split(text, "\n", { plain = true })
  vim.api.nvim_buf_set_lines(bufnr, 0, -1, false, lines)

  vim.bo[bufnr].modifiable = false
end

return M
