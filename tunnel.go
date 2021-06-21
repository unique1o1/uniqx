package main

import (
	"fmt"
	"github.com/gofrs/uuid"
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
		dstUrl:        fmt.Sprintf("http://127.0.0.1:%s", *port),
		dstWSUrl:      fmt.Sprintf("ws://127.0.0.1:%s", *port),
		host:          message.Host,
		token:         message.Token,
		conn:          c,
		socketTracker: make(map[uuid.UUID]chan *ResponseMessage),
	}
	keepAlive(client, time.Minute)
	for {
		message, err := ReadMessage(c)
		if err != nil {
			fmt.Println(err)
			continue
		}
		if value, ok := message.Header["Upgrade"]; ok && (value[0] == "websocket") {
			fmt.Println("sucker")
			ch := make(chan *ResponseMessage)
			client.socketTracker[message.ID] = ch
			go client.wsProcess(message)
		} else if ch, ok := client.socketTracker[message.ID]; ok {
			ch <- message
		} else {
			go client.process(message)
		}

	}
}
