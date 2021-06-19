package main

import (
	"bytes"
	"fmt"
	"github.com/gofrs/uuid"
	"github.com/gorilla/websocket"
	"gopkg.in/mgo.v2/bson"
	"io"
	"net/http"
)

type HandshakeResponseMessage struct {
	Host  string `bson:"host"`
	Token string `bson:"token"`
}
type ResponseMessage struct {
	ID           uuid.UUID            `bson:"id"`
	Method       string               `bson:"method"`
	URL          string               `bson:"url"`
	Body         []byte               `bson:"body"`
	Header       map[string]string    `bson:"header"`
	ResponseChan chan ResponseMessage `bson:"-"`
}

type RequestMessage struct {
	RequestId uuid.UUID         `bson:"request_id"`
	Token     string            `bson:"token"`
	Body      []byte            `bson:"body"`
	Status    int               `bson:"status"`
	Header    map[string]string `bson:"header"`
}

type Client struct {
	host  string
	token string
	conn  *websocket.Conn
}

func (c Client) process(message *ResponseMessage) {
	client := http.DefaultClient
	req, err := http.NewRequest(message.Method, JoinURL(c.host, message.URL), bytes.NewBuffer(message.Body))
	if err != nil {
		fmt.Println(err)
	}
	for key, value := range message.Header {
		req.Header.Add(key, value)
	}
	resp, err := client.Do(req)
	if err != nil {
		fmt.Println(err)
		return
	}
	defer resp.Body.Close()
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return
	}
	reqMessage, _ := bson.Marshal(&RequestMessage{
		RequestId: message.ID,
		Token:     c.token,
		Body:      body,
		Status:    resp.StatusCode,
		Header: func() map[string]string {
			m := make(map[string]string)
			for key := range resp.Header {
				m[key] = resp.Header.Get(key)
			}
			return m
		}(),
	})
	_ = c.conn.WriteMessage(websocket.BinaryMessage, reqMessage)

}
