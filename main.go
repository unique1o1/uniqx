package main

import (
	"flag"
	"fmt"
	"log"
	"os"
	"os/signal"
	"syscall"
)

const VERSION = "v1.1.2"

var port = flag.String("port", "", "Port number of the local server")
var subdomain = flag.String("subdomain", "", "Name for Sub-domain")
var host = flag.String("host", "", "Host of the remote tunnel server")

var v = flag.Bool("version", false, "Show binary's version")

func init() {
	flag.Parse()
	log.SetFlags(log.LstdFlags | log.Lshortfile)

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
	if *v {
		fmt.Println("Version: ", VERSION)
		return
	}
	if *port == "" {
		fmt.Println("Please specify argument port i.e -port 8000")
		return
	}
	if *host == "" {
		fmt.Println("Please specify argument baseUrl i.e -baseUrl example.com")
		return
	}
	fmt.Printf("\033[34m \nPress Ctrl+C to quit.\n")
	openTunnel()

}
