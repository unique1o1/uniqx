package main

import (
	"fmt"
	"github.com/gorilla/websocket"
	"gopkg.in/mgo.v2/bson"
	"path"
	"strings"
	"time"
)

func keepAlive(c *Client, timeout time.Duration) {
	lastResponse := time.Now()
	c.conn.SetPongHandler(func(msg string) error {
		lastResponse = time.Now()
		return nil
	})

	go func() {
		for {
			err := c.WriteMessage(websocket.PingMessage, &RequestMessage{})
			if err != nil {
				return
			}
			time.Sleep(timeout / 2)
			if time.Since(lastResponse) > timeout {
				c.conn.Close()
				return
			}
		}
	}()
}

func ReadHandshakeMessage(conn *websocket.Conn) (*HandshakeResponseMessage, error) {
	resp := new(HandshakeResponseMessage)
	_, message, err := conn.ReadMessage()
	if err != nil {
		return nil, err
	}
	err = bson.Unmarshal(message, resp)
	if err != nil {
		return nil, err

	}
	return resp, nil
}
func ReadMessage(conn *websocket.Conn) (*ResponseMessage, error) {

	resp := new(ResponseMessage)
	_, message, err := conn.ReadMessage()

	if err != nil {
		return nil, err
	}
	err = bson.Unmarshal(message, resp)
	if err != nil {
		return nil, err
	}
	return resp, nil
}
func JoinURL(base string, paths ...string) string {
	p := path.Join(paths...)
	return fmt.Sprintf("%s/%s", strings.TrimRight(base, "/"), strings.TrimLeft(p, "/"))
}
