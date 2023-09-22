
# UNIQX

A simple HTTP/TCP tunnel in Rust that exposes local ports to a remote server, bypassing standard NAT connection firewalls.

## Installation

### Using Rust 

```bash
cargo install --git https://github.com/unique1o1/uniqx.git 
```

## Using pre-built binary

```bash
curl --proto '=https' --tlsv1.2 -sSf https://yunik.com.np/installer.sh \
    | sudo bash -s -- --repo unique1o1/uniqx --to /usr/local/bin
```

## Detailed Usage

This section describes detailed usage for the `uniqx` CLI command.

### Local Forwarding

You can forward a port on your local machine by using the `uniqx` command. This takes a positional argument, the local port to forward, as well as a mandatory `--remote-host` option, which specifies the address of the remote server, and a `--subdomain` option.

### HTTP
```bash
uniqx client http --remote-host example.com --local-port 9000 --subdomain unique
```
For enable console UI
```bash
uniqx client http --remote-host example.com --local-port 9000 --subdomain unique --console
```

### TCP
```bash
uniqx client tcp --port 61589 --remote-host example.com --local-port 5432 --subdomain db
```

In the case of `TCP` you can pass in a `--port` option to pick a specific port on the remote to expose, although the command will fail if this port is not available. Also, passing `--local-host` allows you to expose a different host on your local area network besides the loopback address `localhost`.

The full options are shown below using --help option.

```bash
uniqx client --help
```

## Deploy your own UNIQ tunnel server
You have to deploy your own tunnel server for the client to work.

```bash
uniqx server --domain "example.com"
```

That's all it takes! After the server starts running at a given address, you can then update the `uniqx` command with option `--remote-host <ADDRESS>` to forward a local port to this remote server.

The full options for the `bore server` command are shown below.


```bash
uniqx server --help
```

### Update Uniqx

```bash
sudo uniqx update
```

---
The uniqx tool has an implicit control port at `9876` that is used for creating new connections on demand. When the client initializes a connection, it sends a message to the server on the TCP control port, asking to proxy a selected protocol and remote port(for TCP). The server then responds with an acknowledgement and begins listening for external HTTP/TCP connections.

