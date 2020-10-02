htps

---

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

