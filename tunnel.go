package main

import (
	"fmt"
	"github.com/gorilla/websocket"
	"net/url"
	"os"
	"os/user"
	"time"
)

func getParams() string {
	params := url.Values{}
	params.Add("username",
		func() string {
			if *subdomain == "" {
				username, _ := user.Current()
				return username.Username
			}
			return *subdomain
		}())
	params.Add("port", *port)
	return params.Encode()
}
func openTunnel() {

	u := url.URL{Scheme: "wss", Host: *host, Path: "/_ws/", RawQuery: getParams()}
	fmt.Printf("\u001B[34mConnecting to %s \n\n", u.String())
	c, _, err := websocket.DefaultDialer.Dial(u.String(), nil)

	if err != nil {
		fmt.Println("dial:", err)
	}

	defer c.Close()

	message, err := ReadHandshakeMessage(c)
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
	//fmt.Printf("\u001B[31m Your are now online at: https://%s \n\n", message.Host)
	fmt.Printf("\033[32m%-25s Online\033[00m \n", "Tunnel Status")
	fmt.Printf("%-25s https://%s -> http://127.0.0.1:%s\n", "Forwarded", message.Host, *port)
	fmt.Printf("%-25s http://%s -> http://127.0.0.1:%s \n\n", "Forwarded", message.Host, *port)
	client := &Client{
		host:  fmt.Sprintf("http://127.0.0.1:%s", *port),
		token: message.Token,
		conn:  c,
	}
	keepAlive(client, time.Minute)

	for {
		message, err := ReadMessage(c)
		if err != nil {
			fmt.Println(err)
			continue
		}
		go client.process(message)
	}
}
