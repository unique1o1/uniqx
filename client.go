package main

import (
	"bytes"
	"fmt"
	"github.com/gofrs/uuid"
	"github.com/gorilla/websocket"
	"gopkg.in/mgo.v2/bson"
	"io"
	"log"
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
	SocketMsgType int            `bson:"socket_msg_type,omitempty"`
	ID            uuid.UUID      `bson:"id,omitempty"`
	Method        string         `bson:"method,omitempty"`
	URL           string         `bson:"url,omitempty"`
	Body          []byte         `bson:"body,omitempty"`
	Header        http.Header    `bson:"header,omitempty"`
	Cookie        []*http.Cookie `bson:"cookie,omitempty"`
}

type RequestMessage struct {
	RequestId     uuid.UUID      `bson:"request_id,omitempty"`
	SocketMsgType int            `bson:"socket_msg_type,omitempty"`
	Token         string         `bson:"token,omitempty"`
	Body          []byte         `bson:"body,omitempty"`
	Status        int            `bson:"status,omitempty"`
	Header        http.Header    `bson:"header,omitempty"`
	Cookie        []*http.Cookie `bson:"cookie,omitempty"`
}

type Client struct {
	dstUrl        string
	dstWSUrl      string
	host          string
	token         string
	conn          *websocket.Conn
	socketTracker map[uuid.UUID]chan *ResponseMessage
	sync.Mutex
}

func (c *Client) WriteMessage(messageType int, data []byte) error {
	c.Lock()
	defer c.Unlock()
	return c.conn.WriteMessage(messageType, data)
}

func (c *Client) wsProcess(message *ResponseMessage) {
	socketId := message.ID
	u, _ := url.Parse(JoinURL(c.dstWSUrl, message.URL))
	log.Println(u)
	requestHeader := http.Header{}

	{
		// Pass headers from the incoming request to the dialer to forward them to
		// the final destinations.
		if origin := message.Header.Get("Origin"); origin != "" {
			log.Println(origin)
			requestHeader.Add("Origin", origin)
		}
		for _, prot := range message.Header[http.CanonicalHeaderKey("Sec-WebSocket-Protocol")] {
			requestHeader.Add("Sec-WebSocket-Protocol", prot)
		}
		for _, cookie := range message.Header[http.CanonicalHeaderKey("Cookie")] {
			log.Println(cookie)
			requestHeader.Add("Cookie", cookie)
		}

		// Set the originating protocol of the incoming HTTP request. The SSL might
		// be terminated on our site and because we doing proxy adding this would
		// be helpful for applications on the backend.
		requestHeader.Set("X-Forwarded-Proto", "http")
	}
	dstConn, resp, err := websocket.DefaultDialer.Dial(u.String(), requestHeader)

	if err != nil {
		log.Println("dial:", err)
		return
	}
	defer dstConn.Close()
	reqMessage, _ := bson.Marshal(&RequestMessage{
		Header:    resp.Header,
		RequestId: message.ID,
		Token:     c.token,
	})
	if err != nil {
		return
	}
	c.WriteMessage(websocket.BinaryMessage, reqMessage)
	go func() {
		for {
			msgType, msg, err := dstConn.ReadMessage()
			if err != nil {
				m := websocket.FormatCloseMessage(websocket.CloseNormalClosure, fmt.Sprintf("%v", err))
				if e, ok := err.(*websocket.CloseError); ok {
					if e.Code != websocket.CloseNoStatusReceived {
						m = websocket.FormatCloseMessage(e.Code, e.Text)
					}
				}
				log.Println(err)
				reqMessage, _ := bson.Marshal(&RequestMessage{
					RequestId:     message.ID,
					Token:         c.token,
					SocketMsgType: msgType,
					Body:          m,
				})
				c.WriteMessage(websocket.CloseMessage, reqMessage)
				break
			}
			reqMessage, _ := bson.Marshal(&RequestMessage{
				RequestId:     message.ID,
				Token:         c.token,
				SocketMsgType: msgType,
				Body:          msg,
			})
			err = c.WriteMessage(websocket.BinaryMessage, reqMessage)
			if err != nil {
				log.Println(err)
				break
			}
		}

	}()
	go func() {
		for message = range c.socketTracker[socketId] {
			log.Println(string(message.Body))

			err = dstConn.WriteMessage(message.SocketMsgType, message.Body)
			if err != nil {
				log.Println(err)
				break
			}
		}

	}()

}
func (c *Client) process(message *ResponseMessage) {
	jar, _ := cookiejar.New(&cookiejar.Options{})
	website, _ := url.Parse(JoinURL(c.dstUrl, message.URL))
	jar.SetCookies(
		website,
		message.Cookie,
	)
	client := http.Client{
		Jar: jar,
	}

	req, _ := http.NewRequest(message.Method, JoinURL(c.dstUrl, message.URL), bytes.NewBuffer(message.Body))
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
