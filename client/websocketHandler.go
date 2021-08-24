package client

import (
	"errors"
	"github.com/gorilla/websocket"
	"github.com/pterm/pterm"
	"github.com/unique1o1/jprq/client/model"
	"github.com/unique1o1/jprq/pkg/socket"
	"github.com/unique1o1/jprq/pkg/utils"
	"log"
	"net/http"
	"net/url"
)

/*
WriteToHost

 Read data from JPRQ Server and forwards them to the host server
 JPRQ Server ----->>> WriteToHost() ------->>> host server
*/
func WriteToHost(t *JPRQClient, dstConn *socket.Socket, message *model.ResponseMessage, errChan chan error) {

	for message = range t.SocketTracker[message.ID] {
		if message.Status == -1 {
			errChan <- errors.New("frontend connection closed")
			break
		}
		err := dstConn.WriteMessage(message.SocketMsgType, message.Body)
		if err != nil {
			errChan <- errors.New("connection closed")
			break
		}
	}
}

/*
ReadFromHost

 Read data from Host Server and forwards them to the JPRQ server
 Host Server ----->>> ReadFromHost() ------->>> JPRQ server
*/
func ReadFromHost(t *JPRQClient, dstConn *socket.Socket, message *model.ResponseMessage, errChan chan error) {
	notifyConnectionClose := func(err error) {
		errChan <- err
		pterm.DefaultCenter.Println(pterm.Magenta("Socket connection closed"))
		t.WriteMessage(websocket.CloseMessage, &model.RequestMessage{
			RequestId: message.ID,
			Token:     t.Token,
			Status:    -1, // -1 signals JPRQ Server that the host server socket connection has been broken
			Body:      []byte("connection closed"),
		})
	}
	for {
		msgType, msg, err := dstConn.Conn.ReadMessage()
		if err != nil {
			//if _, ok := err.(*websocket.CloseError); ok {
			//websocket.CloseAbnormalClosure is called when server process exits or websocket.close() is called
			notifyConnectionClose(err)
			//}
			break
		}
		err = t.WriteMessage(websocket.BinaryMessage, &model.RequestMessage{
			RequestId:     message.ID,
			Token:         t.Token,
			SocketMsgType: msgType,
			Body:          msg,
		})
		if err != nil {
			notifyConnectionClose(err)
			break
		}
	}

}

func (t *JPRQClient) WebsocketRequestListener(message *model.ResponseMessage) {
	{
		//website, _ := url.Parse(utils.JoinURL(t.DstUrl, model.URL))
		utils.PrettyPrintRequest(101, message.Method, message.URL)
	}
	socketId := message.ID
	destinationServerURL, _ := url.Parse(utils.JoinURL(t.DstWSUrl, message.URL))
	requestHeader := http.Header{}

	{
		// Pass headers from the incoming request to the dialer to forward them to
		// the final destinations.
		if origin := message.Header.Get("Origin"); origin != "" {
			requestHeader.Add("Origin", origin)
		}
		for _, prot := range message.Header[http.CanonicalHeaderKey("Sec-WebSocket-Protocol")] {
			requestHeader.Add("Sec-WebSocket-Protocol", prot)
		}
		for _, cookie := range message.Header[http.CanonicalHeaderKey("Cookie")] {
			requestHeader.Add("Cookie", cookie)
		}

		// Set the originating protocol of the incoming HTTP request. The SSL might
		// be terminated on our site and because we doing proxy adding this would
		// be helpful for applications on the backend.
		requestHeader.Set("Host", t.Host)
		requestHeader.Set("X-Forwarded-Proto", "http")
		requestHeader.Set("User-Agent", message.Header.Get("User-Agent"))
	}
	//log.Println(requestHeader)
	conn, resp, err := websocket.DefaultDialer.Dial(destinationServerURL.String(), requestHeader)
	if err != nil {
		t.WriteMessage(websocket.BinaryMessage, &model.RequestMessage{
			RequestId: message.ID,
			Token:     t.Token,
			Status:    -1,
		})
		log.Println("dial:", err)
		return
	}
	defer conn.Close()
	defer close(t.SocketTracker[socketId])
	defer delete(t.SocketTracker, socketId)
	hostConn := &socket.Socket{Conn: conn}
	//utils.KeepAlive(hostConn, time.Second*30)
	t.WriteMessage(websocket.BinaryMessage, &model.RequestMessage{
		Header:    resp.Header,
		RequestId: message.ID,
		Token:     t.Token,
	}) // signal JPRQ server that the websocket connection to the host has established
	errChan := make(chan error, 1)
	{
		go WriteToHost(t, hostConn, message, errChan)
		go ReadFromHost(t, hostConn, message, errChan)
	}
	<-errChan
	//pterm.Println(pterm.Magenta("Websocket Connection to host server closed "))
}
