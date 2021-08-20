package main

import (
	"fmt"
	"github.com/gorilla/websocket"
	"gopkg.in/mgo.v2/bson"
	"net/http"
	"path"
	"strings"
	"time"
)

func keepAlive(ws *Socket, timeout time.Duration) {
	lastResponse := time.Now()
	ws.SetPongHandler(func(msg string) error {
		//log.Println("pong..")
		lastResponse = time.Now()
		return nil
	})
	go func() {
		for {
			err := ws.WriteMessage(websocket.PingMessage, []byte("ping"))
			if err != nil {
				fmt.Println("error pinging")
				return
			}
			time.Sleep(timeout)
			if time.Since(lastResponse) > timeout {
				fmt.Println("im closing connections bro")
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
	if len(paths) != 0 && strings.HasSuffix(paths[len(paths)-1], "/") {
		p = p + "/"
	}
	return fmt.Sprintf("%s/%s", strings.TrimRight(base, "/"), strings.TrimLeft(p, "/"))
}

var asciiText = map[string]string{
	"GREEN":  "\u001b[32m",
	"YELLOW": "\u001b[33m",
	"RED":    "\u001b[31m",
	"BLUE":   "\u001b[34m",
	"CYAN":   "\u001b[36m",
	"WHITE":  "\u001b[37m",
}

func AsciiText(code int) string {
	if 100 <= code && code < 200 {
		return asciiText["CYAN"]
	} else if 200 <= code && code < 300 {
		return asciiText["GREEN"]
	} else if 300 <= code && code < 400 {
		return asciiText["BLUE"]

	} else if 400 <= code && code < 500 {
		return asciiText["YELLOW"]

	} else if 500 <= code && code < 600 {
		return asciiText["RED"]

	}
	return asciiText["WHITE"]

}

func PrettyPrintRequest(statusCode int, method, path string) {
	fmt.Printf("%-20s %-110s %s %d [%s] \033[0m \n", method, path, AsciiText(statusCode), statusCode, http.StatusText(statusCode))

}
