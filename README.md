# Ask Me Everything [![CircleCI](https://circleci.com/gh/HowNetWorks/ama.svg?style=shield)](https://circleci.com/gh/HowNetWorks/ama)

## Endpoints

### Who am I?

```console
$ curl https://$SERVER/ama/whoami/
{
  "data": {
    "ip": "185.38.2.2",
    "name": "nat2.panoulu.net"
  }
}
```

### Resolve DNS name (PTR query)

```console
$ curl https://$SERVER/ama/reverse/185.38.2.1
{
  "data": {
    "ip": "185.38.2.1",
    "name": "nat1.panoulu.net"
  }
}
```

### What Cymry knows about IP?

```console
$ curl https://$SERVER/ama/cymru/185.38.2.3
{
  "data": [
    {
      "ip_addr": "185.38.2.3",
      "bgp_prefix": "185.38.0.0/22",
      "as_number": 47605,
      "as_name": "FNE-AS , FI",
      "country_code": "FI",
      "registry": "ripencc",
      "allocated": "2013-10-17"
    }
  ]
}
```

## Errors

Errors are reported with non-OK HTTP response (status code 400-499)
with body containing JSON object with "errors". This tries to mimic
what [GitHub API v3](https://developer.github.com/v3/#client-errors)
does.

```console
$ curl https://$SERVER/ama/cymru/185.38.2
{
  "message": "invalid IP address syntax"
}
```
