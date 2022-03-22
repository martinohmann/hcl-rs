template {
  source = <<-EOF
  This is valid
  EOF
}

heredoc {
  data = <<-EOF
  data = <<-EOF # This is valid
  EOF
}

