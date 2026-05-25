local M = {}

local RESPONSE_BUF_NAME = "req://response"

local function find_response_buf()
  for _, bufnr in ipairs(vim.api.nvim_list_bufs()) do
    if vim.api.nvim_buf_is_valid(bufnr) and vim.api.nvim_buf_get_name(bufnr) == RESPONSE_BUF_NAME then
      return bufnr
    end
  end
end

local function get_or_create_buf()
  local bufnr = find_response_buf()

  if bufnr then
    return bufnr
  end

  bufnr = vim.api.nvim_create_buf(false, true)

  vim.api.nvim_buf_set_name(bufnr, RESPONSE_BUF_NAME)
  vim.bo[bufnr].buftype = "nofile"
  vim.bo[bufnr].bufhidden = "wipe"
  vim.bo[bufnr].swapfile = false
  vim.bo[bufnr].filetype = "req_response"

  return bufnr
end

local function open_buf(bufnr)
  for _, winid in ipairs(vim.api.nvim_list_wins()) do
    if vim.api.nvim_win_get_buf(winid) == bufnr then
      vim.api.nvim_set_current_win(winid)
      return
    end
  end

  vim.cmd("rightbelow vsplit")
  vim.api.nvim_win_set_buf(0, bufnr)
end

function M.show(text)
  local bufnr = get_or_create_buf()

  open_buf(bufnr)

  vim.bo[bufnr].modifiable = true

  local lines = vim.split(text, "\n", { plain = true })
  vim.api.nvim_buf_set_lines(bufnr, 0, -1, false, lines)

  vim.bo[bufnr].modifiable = false
end

return M
