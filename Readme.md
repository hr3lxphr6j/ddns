# ddns

## Features

- docker
- cloudflare
- ipv6
- TBD

## Example of `ddns.toml`

```toml
[[services]]
name = "cloudflare"
domain = ["foo.xxx.com", "bar.xxx.com"]
[services.config]
zoneId = "xxx"
# Choose one of key+email and token authentication methods
key = "xxx"
email = "xxx"
token = "xxx"
```