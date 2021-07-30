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
email = "xxx"
zoneId = "xxx"
key = "xxx
```