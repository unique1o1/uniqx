package main

import (
	"fmt"
	"github.com/gorilla/websocket"
	"gopkg.in/mgo.v2/bson"
	"log"
	"path"
	"strings"
	"time"
)

func keepAlive(ws *Socket, timeout time.Duration) {
	lastResponse := time.Now()
	ws.SetPongHandler(func(msg string) error {
		log.Println("pngpngpngpn")
		lastResponse = time.Now()
		return nil
	})
	go func() {
		for {
			err := ws.WriteMessage(websocket.PingMessage, []byte("ping"))
			if err != nil {
				return
			}
			time.Sleep(timeout / 2)
			if time.Since(lastResponse) > timeout {
				ws.Close()
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
	if strings.HasSuffix(paths[len(paths)-1], "/") {
		p = p + "/"
	}
	return fmt.Sprintf("%s/%s", strings.TrimRight(base, "/"), strings.TrimLeft(p, "/"))
}
