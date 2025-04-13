# Convoyeur

IRCv3 [FILEHOST](https://soju.im/filehost) extension adapter to external file upload services.

## How it Works

Convoyeur is designed to sit behind the bouncer or server that implements FILEHOST, and proxy
upload requests to other places. It reads several headers to determine how to route an upload request:

- `Soju-Username` or `X-Username`: username to look up in the user-to-host mapping
- `X-Upload-Host`: the identifier of an upload host

Convoyeur uses these headers in fallback-style logic:

1. Use `X-Upload-Host` to select an upload host directly
2. Use `Soju-Username` and find the matching upload host
3. Use `X-Username` and find the matching upload host
4. If no username or upload host is given, or there is no matching upload host found, the default upload
   host is used (if it is defined)

Convoyeur also uses the `Content-Type`, `Content-Disposition` (`filename` parameter), and `Content-Length`
headers in accordance with the FILEHOST specification.

## Configuration

Convoyeur is configured via an [HCL](https://github.com/hashicorp/hcl/blob/main/hclsyntax/spec.md)
file. This is passed to the program via the environment variable `CONVOYEUR_CONF=path/to/config.hcl`.

Logging can be configured via the [`RUST_LOG`](https://docs.rs/env_logger/latest/env_logger/#enabling-logging)
environment variable (default: `RUST_LOG=info`).

### Default configuration

The default configuration includes no configured hosts or users, so it is probably not very useful.

```hcl
bind = "localhost:8069"
default_host = null
upload_limit = 25
```

### Example configuration

A more useful configuration might be:

```hcl
# IP/hostname and port to run the adapter on.
# to listen on both v4 and v6, use a domain that resolves to both, like localhost
bind = "localhost:8069"
# default host to use for users not listed in the users block
default_host = "0x0.st"
# maximum file size (mebibytes)
# note that some sites may have lower upload limits
upload_limit = 100

# mapping of username to file host ID
users {
  foo = "mypaste"
  bar = null
}

# host id is given as the block label
host "mypaste" {
  # kind of upload, options:
  #  - form: multipart form
  kind = "form"
  # url to send uploads to
  url = "https://paste.example.com"
  # for kind="form", the form field to use for the file
  file_field = "file"
  # additional fields to send (field_name = contents)
  fields = {
    extra_value = "24"
  }
  # extra headers to add to the request (header_name = contents)
  headers = {
    Authorization = "myS3cr3tT0k3n"
  }
}
```

### Known working file upload services

- [0x0.st](https://0x0.st)
```hcl
host "0x0.st" {
  kind = "form"
  url = "https://0x0.st"
  file_field = "file"
  fields = {
    # optional
    expires = "24"
    secret = "yes"
  }
}
```
- [x0.at](https://x0.at)
```hcl
host "x0.at" {
    kind = "form"
    url = "https://x0.at"
    file_field = "file"
    fields = {
        id_length = "10"
    }
}
```
- [logpaste](https://github.com/mtlynch/logpaste)
  (examples: [logpaste.com](https://logpaste.com), [paste.gentoo.zip](https://paste.gentoo.zip))
```hcl
host "logpaste" {
    kind = "form"
    url = "https://logpaste.com"
    file_field = "_"
}
```
- [rustypaste](https://github.com/orhun/rustypaste)
```hcl
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

```

## Soju Configuration

To use this with the [soju](https://soju.im) bouncer, add the following to your soju configuration:

- a `listen` directive for HTTP (`https://`, `http+unix://`, or `http+insecure://`), for clients to upload to
- an `http-ingress` directive matching the HTTP `listen` directive
- a `file-upload` directive pointing at convoyeur

For example:
```
listen https://:6680
http-ingress http://:6680
file-upload http http://127.0.0.1:8069
```
