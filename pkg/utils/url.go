package utils

import (
	"fmt"
	"net/url"
	"os/user"
	"path"
	"strings"
)

func JoinURL(base string, paths ...string) string {
	p := path.Join(paths...)
	if len(paths) != 0 && strings.HasSuffix(paths[len(paths)-1], "/") {
		p = p + "/"
	}
	return fmt.Sprintf("%s/%s", strings.TrimRight(base, "/"), strings.TrimLeft(p, "/"))
}

func GetParams(subdomain string) string {
	params := url.Values{}
	params.Add("username",
		func() string {
			if subdomain == "" {
				username, _ := user.Current()
				return username.Username
			}
			return subdomain
		}())
	return params.Encode()
}
