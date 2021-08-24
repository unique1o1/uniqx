package utils

import "github.com/pterm/pterm"

type CTablePrinter struct {
	*pterm.TablePrinter
}

func NewCTablePrinter(t *pterm.TablePrinter) *CTablePrinter {
	return &CTablePrinter{TablePrinter: t}
}
func (p CTablePrinter) Render() *CTablePrinter {
	s, _ := p.Srender()
	pterm.DefaultCenter.Println(s)
	return nil
}
