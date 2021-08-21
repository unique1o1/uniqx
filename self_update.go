package main

import (
	"encoding/json"
	"fmt"
	"github.com/kardianos/osext"
	"io"
	"log"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
)

func download(filename, tag string) {

	url := "https://github.com/unique1o1/jprq-go-client/releases/download/%s/%s"
	url = fmt.Sprintf(url, tag, filename)
	fmt.Println(url)
	// Create blank file
	file, err := os.Create(filepath.Join("/tmp", filename))
	if err != nil {
		log.Fatal(err)
	}
	client := http.Client{
		CheckRedirect: func(r *http.Request, via []*http.Request) error {
			r.URL.Opaque = r.URL.Path
			return nil
		},
	}
	// Put content on file
	resp, err := client.Get(url)
	if err != nil {
		log.Fatal(err)
	}
	defer resp.Body.Close()

	size, err := io.Copy(file, resp.Body)

	defer file.Close()

	fmt.Printf("Downloaded a file %s with size %d", filename, size)

}
func update() {
	tagUrl := "https://github.com/unique1o1/jprq/releases/latest"
	binaryPath, _ := osext.Executable()
	fmt.Println(binaryPath)
	req, _ := http.NewRequest("GET", tagUrl, nil)
	req.Header = http.Header{"Accept": []string{"application/json"}}
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		fmt.Println("Failed upgrading..")
		os.Exit(0)

	}
	defer resp.Body.Close()
	body, err := io.ReadAll(resp.Body)
	m := make(map[string]string)
	json.Unmarshal(body, &m)
	var filename string
	switch runtime.GOOS {
	case "darwin":
		filename = "jprq_darwin_amd64.tar.gz"
	case "linux":
		filename = "jprq_linux_amd64.tar.gz"
	case "windows":
		filename = "jprq_windows_amd64.tar.gz"

	}
	download(filename, m["tag_name"])
	cmd := exec.Command("tar", "-xf", filepath.Join("/tmp/", filename), "-C", "/tmp")
	err = cmd.Run()
	fmt.Println("taring")

	if err != nil {
		fmt.Println(err)
		os.Exit(0)
	}
	fmt.Println("removing")
	cmd = exec.Command("rm", "-f", binaryPath)
	err = cmd.Run()
	if err != nil {
		fmt.Println(err)
		os.Exit(0)

	}
	fmt.Println("copyin")
	cmd = exec.Command("sudo", "cp", filepath.Join("/tmp/", "jprq"), binaryPath)
	err = cmd.Run()
	if err != nil {
		fmt.Println(err)
		os.Exit(0)
	}
	os.Exit(0)
}
