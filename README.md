# JPRQ - Ngrok Alternative

## Get Your Localhost Online and HTTPS


## How JPRQ is different from Ngrok?

- JPRQ is a free and open-source Ngrok alternative to expose local servers online easily.
- It allows developers to serve unlimited requests to the local server compared to Ngrok's **40 requests/minute** limit.
- It can expose multiple ports at the same time compared to Ngrok with **1 port** limit.

---
## **Note**: 
This client doesn't work with the original [jprq.io](https://github.com/azimjohn/jprq/) server (i.e open.jprq.live)

## Deploy your own jprq tunnel server
You have to deploy your own tunnel server for the client to work. Visit [jprq.io](https://github.com/unique1o1/jprq.io)

---
## How to install

### Using Go
```bash
$ go install github.com/unique1o1/jprq@latest
```
## Using pre-built binary
```bash
$ curl -f https://raw.githubusercontent.com/unique1o1/jprq/main/install.sh | sudo sh
```
## How to use

Replace 8000 with the port you want to expose and replace host with your tunnel server domain

```
$ jprq -port  8000 -host example.com
```

Press Ctrl+C to stop it

## Custom Subdomain

Replace `subdomain` with a subdomain you want

```
$ jprq -port 8000 -subdomain=subdomain -host example.com
```

## What's New
- websocket support added (fire up your jupyter notebook in no time)

## How JPRQ Works

<img alt width="100%" src="https://i.imgur.com/1kXPzyd.png">

---

### JPRQ's Server-side implementation in Golang:

<a href="https://github.com/unique1o1/jprq.io">https://github.com/unique1o1/jprq.io</a>

## Limitations

- Doesn't work with HTTP Polling

