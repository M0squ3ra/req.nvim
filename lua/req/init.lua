local M = {}

function M.run(opts)
  require("req.runner").run(opts)
end

function M.curl(opts)
  require("req.runner").curl(opts)
end

function M.curl_copy(opts)
  require("req.runner").curl_copy(opts)
end

function M.rerun()
  require("req.runner").rerun()
end

function M.hello()
  require("req.runner").run()
end

return M
