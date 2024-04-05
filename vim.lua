-- with lspconfig
--
require('lspconfig.configs').lls = nil

require('lspconfig.configs').llsdev = {
  default_config = {
    cmd = { 'target/debug/lls', '--stdio' },
    filetypes = { 'text', 'markdown' },
    root_dir = function(_)
      return '/'
    end,
  },
}
require('lspconfig').llsdev.setup {
  init_options = { wordnet = os.getenv("WNSEARCHDIR") },
}

-- or without lspconfig
--
-- vim.lsp.start({
--   name = 'lls',
--   cmd = { 'target/debug/lls' },
--   root_dir = '.',
--   init_options = { wordnet = os.getenv("WNSEARCHDIR") },
-- })

vim.lsp.set_log_level("DEBUG")
vim.keymap.set('n', 'K', vim.lsp.buf.hover, {noremap = true})
vim.keymap.set('n', 'gd', vim.lsp.buf.definition, {noremap = true})
vim.keymap.set('n', 'ga', vim.lsp.buf.code_action, {noremap = true})
