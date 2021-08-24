package model

import (
	"github.com/gofrs/uuid"
	"net/http"
)

type RequestMessage struct {
	RequestId     uuid.UUID      `bson:"request_id,omitempty"`
	SocketMsgType int            `bson:"socket_msg_type,omitempty"`
	Token         string         `bson:"token,omitempty"`
	Body          []byte         `bson:"body,omitempty"`
	Status        int            `bson:"status,omitempty"`
	Header        http.Header    `bson:"header,omitempty"`
	Cookie        []*http.Cookie `bson:"cookie,omitempty"`
}
