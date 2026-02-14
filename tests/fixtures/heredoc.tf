resource "aws_iam_policy" "example" {
  policy = <<EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": "s3:GetObject",
      "Resource": "arn:aws:s3:::${var.bucket}/*"
    }
  ]
}
EOF
}

resource "aws_instance" "indented" {
  user_data = <<-SCRIPT
    #!/bin/bash
    echo "Hello ${var.name}"
    SCRIPT
}
