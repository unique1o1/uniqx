package main

import (
	"bytes"
	"flag"
	"fmt"
	"github.com/gofrs/uuid"
	"github.com/gorilla/websocket"
	"gopkg.in/mgo.v2/bson"
	"io"
	"log"
	"net/http"
	"net/url"
	"os"
	"os/signal"
	"os/user"
	"path"
	"strings"
	"syscall"
)

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

type HandshakeResponseMessage struct {
	Host  string `bson:"host"`
	Token string `bson:"token"`
}
type Client struct {
	host  string
	token string
	conn  *websocket.Conn
}

func JoinURL(base string, paths ...string) string {
	p := path.Join(paths...)
	return fmt.Sprintf("%s/%s", strings.TrimRight(base, "/"), strings.TrimLeft(p, "/"))
}

func (c Client) process(message *ResponseMessage) {
	println("got request")
	client := http.DefaultClient
	req, err := http.NewRequest(message.Method, JoinURL(c.host, message.URL), bytes.NewBuffer(message.Body))
	if err != nil {
		log.Println(err)
	}
	for key, value := range message.Header {
		req.Header.Add(key, value)
	}
	resp, err := client.Do(req)
	if err != nil {
		log.Println(err)
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

var port = flag.String("port", "", "Port number of the local server")
var subdomain = flag.String("subdomain", "", "Name for Sub-domain")
var host = flag.String("host", "open.yunik.com.np", "Host of the remote server")

func ReadMessage(conn *websocket.Conn) (*ResponseMessage, error) {

	resp := new(ResponseMessage)

	_, message, err := conn.ReadMessage()
	if err != nil {
		log.Println(err)
		return nil, err
	}
	err = bson.Unmarshal(message, resp)
	if err != nil {
		log.Println(err)
		return nil, err

	}
	return resp, nil
}

func ReadHandshakeMessage(conn *websocket.Conn, ) (*HandshakeResponseMessage, error) {

	resp := new(HandshakeResponseMessage)
	_, message, err := conn.ReadMessage()
	if err != nil {
		log.Println(err)
		return nil, err
	}
	err = bson.Unmarshal(message, resp)
	if err != nil {
		log.Println(err)
		return nil, err

	}
	return resp, nil
}
func SetupCloseHandler() {
	c := make(chan os.Signal)
	signal.Notify(c, os.Interrupt, syscall.SIGTERM)
	go func() {
		<-c
		fmt.Println("\r- Ctrl+C pressed in Terminal")
		os.Exit(0)
	}()
}
func main() {
	// Setup our Ctrl+C handler
	SetupCloseHandler()

	flag.Parse()
	log.SetFlags(log.LstdFlags | log.Lshortfile)

	fmt.Println("\n\033[1;35mjprq \033[00m \033[34m \n Press Ctrl+C to quit.\n")

	interrupt := make(chan os.Signal)
	signal.Notify(interrupt, os.Interrupt, syscall.SIGTERM)

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
	u := url.URL{Scheme: "wss", Host: *host, Path: "/_ws/", RawQuery: params.Encode()}
	log.Printf("connecting to %s", u.String())
	log.Println(u.String())
	c, _, err := websocket.DefaultDialer.Dial(u.String(), nil)
	if err != nil {
		log.Fatal("dial:", err)
	}
	defer c.Close()
	message, err := ReadHandshakeMessage(c)
	if err != nil {
		log.Println(err)
	}
	log.Println("your url is:", "https://"+message.Host)
	client := Client{
		host:  fmt.Sprintf("http://127.0.0.1:%s", *port),
		token: message.Token,
		conn:  c,
	}

	for {

		message, err := ReadMessage(c)
		if err != nil {
			log.Println(err)
			continue

		}
		go client.process(message)

	}

}
