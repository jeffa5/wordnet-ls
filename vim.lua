-- with lspconfig
--
if require('lspconfig.configs').wordnet ~= nil then
  require('lspconfig.configs').wordnet = nil
end

require('lspconfig.configs').wordnet_dev = {
  default_config = {
    cmd = { 'target/debug/wordnet-ls', '--stdio' },
    filetypes = { 'text', 'markdown' },
    root_dir = function(_)
      return '/'
    end,
  },
}
require('lspconfig').wordnet_dev.setup {
  init_options = { wordnet = os.getenv("WNSEARCHDIR") },
}

-- or without lspconfig
--
-- vim.lsp.start({
--   name = 'wordnet-ls',
--   cmd = { 'target/debug/wordnet-ls' },
--   root_dir = '.',
--   init_options = { wordnet = os.getenv("WNSEARCHDIR") },
-- })

vim.lsp.set_log_level("DEBUG")
vim.keymap.set('n', 'K', vim.lsp.buf.hover, { noremap = true })
vim.keymap.set('n', 'gd', vim.lsp.buf.definition, { noremap = true })
vim.keymap.set('n', 'ga', vim.lsp.buf.code_action, { noremap = true })
