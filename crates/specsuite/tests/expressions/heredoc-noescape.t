// The mixing of QuotedString in the test with the Heredoc output in the results
// is important to this test. It ensures proper escaping for both code paths
result = {
    unescaped_regex = {
        contents = "www\\\\.example\\\\.com\n"
    }

    escaped_regex = {
        contents = <<-EOT
            www\\.example\\.com
            EOT
    }
}