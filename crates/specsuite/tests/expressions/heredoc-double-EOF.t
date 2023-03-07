// Double EOF are valid, as long as only the final is preceded by a newline and whitespace.
result = {
  heredoc = {
    data = "data = <<EOF # This is actually valid\n"
  }
}
