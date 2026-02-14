variable "name" {
  type    = string
  default = "hello"
}

resource "aws_instance" "web" {
  ami           = "ami-12345"
  instance_type = "t2.micro"

  tags = {
    Name = "web-server"
  }
}

output "id" {
  value = aws_instance.web.id
}
