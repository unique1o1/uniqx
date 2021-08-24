package main

import (
	"flag"
	"fmt"
	"github.com/pterm/pterm"
	"github.com/unique1o1/jprq/pkg/utils"
	"log"
	"os"
	"os/signal"
	"syscall"
)

var VERSION = "dev"

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
		fmt.Println("Please specify argument host i.e -host example.com")
		return
	}
	fmt.Printf("\033[34m \nPress Ctrl+C to quit.\n")
	initializeHeader()
	Serve(openTunnel())

}

func initializeHeader() {
	// Generate BigLetters
	utils.ClearConsole()
	p, _ := pterm.DefaultBigText.WithLetters(pterm.NewLettersFromString("JPRQ")).Srender()
	pterm.DefaultCenter.Println(p) // Print BigLetters with the default CenterPrinter
	pterm.DefaultCenter.WithCenterEachLineSeparately().Println("Get Your Localhost Online")

}
