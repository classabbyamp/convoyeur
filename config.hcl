bind = "127.0.0.1:8069"
default_host = "0x0.st"

users {
  foo = "0x0.st"
  bar = "rustypaste"
}

host "0x0.st" {
  kind = "form"
  url = "https://0x0.st"
  file_field = "file"
  fields = {
    expires = "24"
    secret = "yes"
  }
}

host "rustypaste" {
  kind = "form"
  url = "https://rpaste.example.com"
  file_field = "file"
  fields = {
    expire = "1d"
  }
  headers = {
    Authorization = "myS3cr3tT0k3n"
  }
}
