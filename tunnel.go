package main

import (
	"fmt"
	"github.com/gofrs/uuid"
	"github.com/gorilla/websocket"
	"github.com/pterm/pterm"
	"github.com/unique1o1/jprq/client"
	"github.com/unique1o1/jprq/client/model"
	"github.com/unique1o1/jprq/pkg/socket"
	"github.com/unique1o1/jprq/pkg/utils"
	"gopkg.in/mgo.v2/bson"
	"log"
	"net/url"
	"os"
)

func ReadHandshakeMessage(conn *websocket.Conn) (*model.HandshakeResponseMessage, error) {
	resp := new(model.HandshakeResponseMessage)
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
func openTunnel() *client.JPRQClient {

	u := url.URL{Scheme: "wss", Host: *host, Path: "/_ws/", RawQuery: utils.GetParams(*subdomain)}
	pterm.DefaultCenter.Println(pterm.Green("Connecting to Tunnel Server"))
	conn, _, err := websocket.DefaultDialer.Dial(u.String(), nil)

	if err != nil {
		fmt.Println("dial:", err)
		os.Exit(0)
	}

	message, err := ReadHandshakeMessage(conn)
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}

	pterm.DefaultCenter.Println(pterm.Green("Connected to Tunnel Server"))

	t := pterm.DefaultTable.WithBoxed().WithHasHeader().WithData(pterm.TableData{
		{pterm.Green("Tunnel Status"), pterm.Green("Online")},
		{pterm.White("Forwarded"), pterm.White(fmt.Sprintf("https://%s -> http://127.0.0.1:%s", message.Host, *port))},
		{pterm.White("Forwarded"), pterm.White(fmt.Sprintf("http://%s -> http://127.0.0.1:%s", message.Host, *port))},
	})
	utils.NewCTablePrinter(t).Render()

	return &client.JPRQClient{
		DstUrl:        fmt.Sprintf("http://127.0.0.1:%s", *port),
		DstWSUrl:      fmt.Sprintf("ws://127.0.0.1:%s", *port),
		Host:          message.Host,
		Token:         message.Token,
		Conn:          &socket.Socket{Conn: conn},
		SocketTracker: make(map[uuid.UUID]chan *model.ResponseMessage),
	}

}

func Serve(c *client.JPRQClient) {
	defer c.Conn.Close()
	for {
		message, err := c.ReadMessage()
		if err != nil {
			if _, ok := err.(*websocket.CloseError); ok {
				pterm.DefaultCenter.Print(pterm.Red("Server connection closed"))
				break
			}
			log.Println(err)
			break
		}
		if value, ok := message.Header["Upgrade"]; ok && (value[0] == "websocket") {
			// establish new websocket connection
			ch := make(chan *model.ResponseMessage)
			c.SocketTracker[message.ID] = ch
			go c.WebsocketRequestListener(message)
		} else if ch, ok := c.SocketTracker[message.ID]; ok {
			// send model for  existing websocket connection to its own channel
			ch <- message
		} else {
			go c.HTTPRequestListener(message)
		}
	}
}
