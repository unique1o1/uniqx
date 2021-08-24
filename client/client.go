package client

import (
	"github.com/gofrs/uuid"
	"github.com/unique1o1/jprq/client/model"
	"github.com/unique1o1/jprq/pkg/socket"
	"gopkg.in/mgo.v2/bson"
)

type JPRQClient struct {
	DstUrl        string
	DstWSUrl      string
	Host          string
	Token         string
	Conn          *socket.Socket
	SocketTracker map[uuid.UUID]chan *model.ResponseMessage
}

func (t *JPRQClient) ReadMessage() (*model.ResponseMessage, error) {
	resp := new(model.ResponseMessage)
	_, message, err := t.Conn.ReadMessage()

	if err != nil {
		return nil, err
	}
	err = bson.Unmarshal(message, resp)
	if err != nil {
		return nil, err
	}
	return resp, nil
}

func (t *JPRQClient) WriteMessage(messageType int, data *model.RequestMessage) error {
	reqMessage, err := bson.Marshal(data)
	if err != nil {
		return err
	}
	return t.Conn.WriteMessage(messageType, reqMessage)
}

//
