vim.lsp.start({
  name = 'lls',
  cmd = { 'target/release/lls' },
  root_dir = vim.fs.dirname(vim.fs.find({ 'LLS.txt' })[1]),
  init_options = { wordnet = os.getenv("WORDNET") },
})
vim.lsp.set_log_level("DEBUG")
vim.keymap.set('n', 'K', vim.lsp.buf.hover, {noremap = true})
