# htps

hpts(http-proxy-to-socks) is a tool to convert socks proxy into http proxy

## install

> cargo install htps

## usage

> htps --help

```
USAGE:
    hpts [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -v               Sets the level of verbosity
    -V, --version    Prints version information

OPTIONS:
    -p <port>         specify the listening port of http proxy server, default: 8080
    -s <socks>        specify your socks proxy host, default: 127.0.0.1:1080
```

## Why rebuild wheels?

I was using [oyyd/http-proxy-to-socks](https://github.com/oyyd/http-proxy-to-socks), and notice the memory cost is not cheap.
So I implement the rust version, it is very lightweight, with much lower memory-consumption compared to nodejs version.
