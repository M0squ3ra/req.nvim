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

local function append(lines, value)
  table.insert(lines, value)
end

local function append_directives(lines, directives)
  append(lines, "###Directives")

  if not directives or vim.tbl_isempty(directives) then
    append(lines, "None")
  else
    for _, directive in ipairs(directives) do
      append(lines, directive)
    end
  end

  append(lines, "")
end

local function append_response(lines, response)
  local options = config.options.output

  append(lines, "###Response")

  if options.show_status then
    append(lines, "HTTP " .. tostring(response.status))
  end

  if options.show_headers and response.headers then
    for _, header in ipairs(response.headers) do
      append(lines, header.name .. ": " .. header.value)
    end
  end

  if options.show_body then
    append(lines, "")

    if response.body and response.body ~= "" then
      vim.list_extend(lines, vim.split(response.body, "\n", { plain = true }))
    end
  end
end

local function response_lines(payload)
  local options = config.options.output
  local lines = {}

  append(lines, "###Name: " .. payload.request.name)
  append(lines, "")

  if options.show_directives then
    append_directives(lines, payload.request.directives)
  end

  append_response(lines, payload.response)

  return lines
end

function M.show_lines(lines, filetype)
  local bufnr = get_or_create_buf()

  open_buf(bufnr)

  vim.bo[bufnr].filetype = filetype or config.options.output.filetype.response
  vim.bo[bufnr].modifiable = true

  vim.api.nvim_buf_set_lines(bufnr, 0, -1, false, lines)

  vim.bo[bufnr].modifiable = false
end

function M.show(text, filetype)
  M.show_lines(vim.split(text, "\n", { plain = true }), filetype)
end

function M.show_response(payload)
  M.show_lines(response_lines(payload), config.options.output.filetype.response)
end

return M
