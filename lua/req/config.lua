local M = {}

local defaults = {
  window = {
    position = "right",
    size = nil,
    reuse = true,
  },
  output = {
    buffer_name = "req://response",
    filetype = {
      response = "req_response",
    },
  },
  clipboard = {
    register = "+",
  },
}

M.options = vim.deepcopy(defaults)

function M.setup(opts)
  M.options = vim.tbl_deep_extend("force", vim.deepcopy(defaults), opts or {})
end

return M
