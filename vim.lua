vim.lsp.start({
  name = 'lls',
  cmd = { 'target/release/lls' },
  root_dir = '.',
  init_options = { wordnet = os.getenv("WORDNET") },
})
vim.lsp.set_log_level("DEBUG")
vim.keymap.set('n', 'K', vim.lsp.buf.hover, {noremap = true})
vim.keymap.set('n', 'gd', vim.lsp.buf.definition, {noremap = true})
