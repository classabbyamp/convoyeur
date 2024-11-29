rustypaste

- form:
    - `file`: file
- headers:
    - `Authorization`: token
    - `expire`: timeout

0x0.st

- form:
    - `file`: file
    - `expires`: timeout
    - `secret`: content ignored, but gives a longer url

x0.at

- form:
    - `file`: file

paste.gentoo.zip (logpaste)

- form:
    - `_`: file

termbin.com: doesn't accept arbitrary files (png was mangled)

```hcl
bind {
  host = "127.0.0.1"
  port = 8080
}

host "0x0.st" {
  kind = "form"
  url = "https://0x0.st"
  file_field = "file"
  fields = {
    expires = "1d"
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

default_host = "0x0.st"

users {
  foo = "0x0.st"
  bar = "rustypaste"
}
```
