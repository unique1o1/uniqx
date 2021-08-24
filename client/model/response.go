package model

import (
	"github.com/gofrs/uuid"
	"net/http"
)

type HandshakeResponseMessage struct {
	Host  string `bson:"host"`
	Token string `bson:"token"`
}
type ResponseMessage struct {
	Status        int            `bson:"status,omitempty"`
	SocketMsgType int            `bson:"socket_msg_type,omitempty"`
	ID            uuid.UUID      `bson:"id,omitempty"`
	Method        string         `bson:"method,omitempty"`
	URL           string         `bson:"url,omitempty"`
	Body          []byte         `bson:"body,omitempty"`
	Header        http.Header    `bson:"header,omitempty"`
	Cookie        []*http.Cookie `bson:"cookie,omitempty"`
}
