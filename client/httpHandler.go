package client

import (
	"bytes"
	"fmt"
	"github.com/gorilla/websocket"
	"github.com/unique1o1/jprq/client/model"
	"github.com/unique1o1/jprq/pkg/utils"
	"io"
	"net/http"
	"net/http/cookiejar"
	"net/url"
)

func (t *JPRQClient) HTTPRequestListener(message *model.ResponseMessage) {
	website, _ := url.Parse(utils.JoinURL(t.DstUrl, message.URL))
	jar, _ := cookiejar.New(&cookiejar.Options{})
	jar.SetCookies(
		website,
		message.Cookie,
	)
	client := http.Client{
		Jar: jar,
		CheckRedirect: func(req *http.Request, via []*http.Request) error { //stop following redirects
			return http.ErrUseLastResponse
		},
	}
	req, _ := http.NewRequest(message.Method, utils.JoinURL(t.DstUrl, message.URL), bytes.NewBuffer(message.Body))
	req.Host = t.Host
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

	{
		utils.PrettyPrintRequest(resp.StatusCode, message.Method, website.Path)
	}
	reqMessage := &model.RequestMessage{
		RequestId: message.ID,
		Token:     t.Token,
		Body:      body,
		Status:    resp.StatusCode,
		Cookie:    client.Jar.Cookies(website),
		Header:    resp.Header,
	}
	t.WriteMessage(websocket.BinaryMessage, reqMessage)

}
