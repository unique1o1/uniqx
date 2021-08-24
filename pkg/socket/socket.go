package socket

import (
	"github.com/gorilla/websocket"
	"sync"
)

type Socket struct {
	sync.Mutex
	*websocket.Conn
}

func (s *Socket) WriteMessage(messageType int, data []byte) error {
	s.Lock()
	defer s.Unlock()
	return s.Conn.WriteMessage(messageType, data)
}
