---@type table<string, vim.lsp.Config>
return {
    rust_analyzer = {
        settings = {
            ["rust-analyzer"] = {
                check = { command = nil },
            }
        }
    }
}
