# filehost-adapter

IRC [FILEHOST](https://soju.im/filehost) extension adapter to external paste services.

## Configuration

filehost-adapter is configured via an [HCL](https://github.com/hashicorp/hcl/blob/main/hclsyntax/spec.md)
file. This is passed to the program via the environment variable `ADAPTER_CONF=path/to/config.hcl`.

Logging can be configured via the [`RUST_LOG`](https://docs.rs/env_logger/latest/env_logger/#enabling-logging)
environment variable (default: `RUST_LOG=info`).

### Default configuration

The default configuration includes no configured hosts or users, so it is probably not very useful.

```hcl
bind = "localhost:8069"
default_host = null
```

### Example configuration

A more useful configuration might be:

```hcl
# IP/hostname and port to run the adapter on
bind = "localhost:8069"
# default host to use for users not listed in the users block
default_host = "0x0.st"

# mapping of username to file host ID
users {
  foo = "0x0.st"
  bar = "rustypaste"
}

# host id is given as the block label
host "0x0.st" {
  # kind of upload, options:
  #  - form: multipart form
  kind = "form"
  # url to send uploads to
  url = "https://0x0.st"
  # for kind="form", the form field to use for the file
  file_field = "file"
  # additional fields to send (field_name = contents)
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
  # extra headers to add to the request (header_name = contents)
  headers = {
    Authorization = "myS3cr3tT0k3n"
  }
}

```
