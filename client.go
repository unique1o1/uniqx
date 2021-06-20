package main

import (
	"bytes"
	"fmt"
	"github.com/gofrs/uuid"
	"github.com/gorilla/websocket"
	"gopkg.in/mgo.v2/bson"
	"io"
	"net/http"
	"net/http/cookiejar"
	"net/url"
	"sync"
)

type HandshakeResponseMessage struct {
	Host  string `bson:"host"`
	Token string `bson:"token"`
}
type ResponseMessage struct {
	ID     uuid.UUID           `bson:"id"`
	Method string              `bson:"method"`
	URL    string              `bson:"url"`
	Body   []byte              `bson:"body"`
	Header map[string][]string `bson:"header"`
	Cookie []*http.Cookie      `bson:"cookie"`
}

type RequestMessage struct {
	RequestId uuid.UUID           `bson:"request_id"`
	Token     string              `bson:"token"`
	Body      []byte              `bson:"body"`
	Status    int                 `bson:"status"`
	Header    map[string][]string `bson:"header"`
	Cookie    []*http.Cookie      `bson:"cookie"`
}

type Client struct {
	baseUrl string
	host    string
	token   string
	conn    *websocket.Conn
	sync.Mutex
}

func (c *Client) WriteMessage(messageType int, data []byte) error {
	c.Lock()
	defer c.Unlock()
	return c.conn.WriteMessage(messageType, data)
}

func (c *Client) process(message *ResponseMessage) {
	jar, _ := cookiejar.New(&cookiejar.Options{})
	website, _ := url.Parse(JoinURL(c.baseUrl, message.URL))
	jar.SetCookies(
		website,
		message.Cookie,
	)
	client := http.Client{
		Jar: jar,
	}

	req, _ := http.NewRequest(message.Method, JoinURL(c.baseUrl, message.URL), bytes.NewBuffer(message.Body))
	req.Host = c.host
	req.Header = message.Header
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
		Cookie:    client.Jar.Cookies(website),
		Header:    resp.Header,
	})
	_ = c.WriteMessage(websocket.BinaryMessage, reqMessage)

}
