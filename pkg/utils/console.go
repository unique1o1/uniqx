package utils

import (
	"github.com/pterm/pterm"
	"os"
	"os/exec"
)

func StatusWiseColor(code int) string {
	if 100 <= code && code < 200 {
		return pterm.Cyan(code)
	} else if 200 <= code && code < 300 {
		return pterm.Green(code)
	} else if 300 <= code && code < 400 {
		return pterm.Blue(code)

	} else if 400 <= code && code < 500 {
		return pterm.Yellow(code)

	} else if 500 <= code && code < 600 {
		return pterm.Red(code)

	}
	return pterm.White(code)

}
func ClearConsole() {
	cmd := exec.Command("clear")
	cmd.Stdout = os.Stdout
	cmd.Run()
}
func PrettyPrintRequest(statusCode int, method, path string) {
	_ = pterm.DefaultTable.WithHasHeader().WithData(pterm.TableData{
		{pterm.Cyan(method), pterm.White(path), StatusWiseColor(statusCode)},
	}).Render()

}
