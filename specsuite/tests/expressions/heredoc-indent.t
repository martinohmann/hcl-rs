// Heredocs with <<- must strip whitespace, but only what is present before EOF
result = {
  heredoc = {
    data = "Indent was stripped.\n  And it was the correct amount.\n"
  }
}
