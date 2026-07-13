package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

type Input struct {
	Action   string `json:"action"`
	Code     string `json:"code,omitempty"`
	Args     string `json:"args,omitempty"`
	Filename string `json:"filename,omitempty"`
}

type Output struct {
	Stdout   string `json:"stdout"`
	Stderr   string `json:"stderr"`
	ExitCode int    `json:"exit_code"`
	Result   string `json:"result"`
}

func writeOutput(o Output) {
	b, _ := json.Marshal(o)
	fmt.Println(string(b))
}

func execCode(code string) {
	tmpDir, err := os.MkdirTemp("", "klyron-go-*")
	if err != nil {
		writeOutput(Output{Stderr: err.Error(), ExitCode: 1})
		return
	}
	defer os.RemoveAll(tmpDir)

	srcPath := filepath.Join(tmpDir, "main.go")
	if err := os.WriteFile(srcPath, []byte(code), 0644); err != nil {
		writeOutput(Output{Stderr: err.Error(), ExitCode: 1})
		return
	}

	binPath := filepath.Join(tmpDir, "prog")
	cmd := exec.Command("go", "build", "-o", binPath, srcPath)
	if out, err := cmd.CombinedOutput(); err != nil {
		writeOutput(Output{Stderr: string(out), ExitCode: 1, Result: "Compilation failed"})
		return
	}

	cmd = exec.Command(binPath)
	out, err := cmd.CombinedOutput()
	exitCode := 0
	if err != nil {
		if exitErr, ok := err.(*exec.ExitError); ok {
			exitCode = exitErr.ExitCode()
		}
	}
	writeOutput(Output{Stdout: string(out), ExitCode: exitCode, Result: string(out)})
}

func evalExpr(expr string) {
	code := fmt.Sprintf(`package main
import "fmt"
func main() {
	fmt.Print(%s)
}`, expr)
	execCode(code)
}

func main() {
	scanner := bufio.NewScanner(os.Stdin)
	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if line == "" {
			continue
		}

		var input Input
		if err := json.Unmarshal([]byte(line), &input); err != nil {
			writeOutput(Output{Stderr: "Invalid JSON", ExitCode: 1})
			continue
		}

		switch input.Action {
		case "exec", "run":
			execCode(input.Code)
		case "eval":
			evalExpr(input.Code)
		case "ping", "":
			writeOutput(Output{Stdout: "pong", Result: "ok"})
		default:
			writeOutput(Output{Stderr: "Unknown action: " + input.Action, ExitCode: 1})
		}
	}
}
