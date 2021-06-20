package main

import (
	"flag"
	"fmt"
	"os"
	"os/signal"
	"syscall"
)

var port = flag.String("port", "", "Port number of the local server")
var subdomain = flag.String("subdomain", "", "Name for Sub-domain")
var host = flag.String("host", "", "Host of the remote tunnel server")

func init() {
	flag.Parse()
}
func main() {
	go func() {
		//Setup our Ctrl+C handler
		c := make(chan os.Signal)
		signal.Notify(c, os.Interrupt, syscall.SIGTERM)
		go func() {
			<-c
			fmt.Println("\n\033[31mTunnel closed\033[00m")
			os.Exit(0)
		}()
	}()
	if *port == "" {
		fmt.Println("Please specify argument port i.e -port 8000")
		return
	}
	if *host == "" {
		fmt.Println("Please specify argument host i.e -host example.com")
		return
	}
	fmt.Printf("\033[34m \nPress Ctrl+C to quit.\n")
	openTunnel()

}
