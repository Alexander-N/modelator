package main

import (
	"encoding/json"
	"errors"
	"fmt"
	"io/ioutil"
	"os/exec"
)

// From the root of this repository run:
// ```
// cargo install modelator --path modelator/ 
// go run modelator.go
// ```
func main() {
	traces, err := Traces(
		"modelator/tests/integration/tla/NumbersAMaxBMinTest.tla",
		"modelator/tests/integration/tla/Numbers.cfg",
	)
	if err != nil {
		fmt.Println(err)
	} else {
		for _, trace := range traces {
			fmt.Println(string(trace))
		}
	}
}

func Traces(tlaTestsFile string, tlaConfigFile string) ([][]byte, error) {
	var traces [][]byte

	// generate tests
	generatedTests, err := GenerateTests(tlaTestsFile, tlaConfigFile)
	if err != nil {
		return traces, err
	}

	// generate json trace for each test
	for _, generatedTest := range generatedTests {

		// generate tla trace
		tlaTrace, err := Test(generatedTest.TlaFile, generatedTest.TlaConfigFile)
		if err != nil {
			return traces, err
		}

		// convert tla trace to a json trace
		jsonTrace, err := TlaTraceToJsonTrace(tlaTrace.TlaTraceFile)
		if err != nil {
			return traces, err
		}

		// read json trace file
		trace, err := ioutil.ReadFile(jsonTrace.JsonTraceFile)
		if err != nil {
			return traces, err
		}
		traces = append(traces, trace)
	}

	return traces, nil
}

func GenerateTests(tlaTestsFile string, tlaConfigFile string) ([]GeneratedTest, error) {
	var generatedTests []GeneratedTest
	result, err := Modelator("tla", "generate-tests", tlaTestsFile, tlaConfigFile)
	if err == nil {
		json.Unmarshal(result, &generatedTests)
	}
	return generatedTests, err
}

func Test(tlaTestsFile string, tlaConfigFile string) (TlaTrace, error) {
	var tlaTrace TlaTrace
	result, err := Modelator("tlc", "test", tlaTestsFile, tlaConfigFile)
	if err == nil {
		json.Unmarshal(result, &tlaTrace)
	}
	return tlaTrace, err
}

func TlaTraceToJsonTrace(tlaTraceFile string) (JsonTrace, error) {
	var jsonTrace JsonTrace
	result, err := Modelator("tla", "tla-trace-to-json-trace", tlaTraceFile)
	if err == nil {
		json.Unmarshal(result, &jsonTrace)
	}
	return jsonTrace, err
}

func Modelator(modelatorModule string, modelatorMethod string, args ...string) (json.RawMessage, error) {
	allArgs := append([]string{modelatorModule, modelatorMethod}, args...)
	cmd := exec.Command("modelator", allArgs...)

	// run command
	output, err := cmd.Output()
	fmt.Print("output: ", string(output))
	fmt.Println("error: ", err)

	// parse its output
	var modelatorOutput ModelatorOutput
	json.Unmarshal(output, &modelatorOutput)

	if modelatorOutput.Status == "error" {
		return nil, errors.New(string(modelatorOutput.Result))
	} else if modelatorOutput.Status == "success" {
		return modelatorOutput.Result, nil
	} else {
		panic("[modelator] unexpected status: " + modelatorOutput.Status)
	}
}

type ModelatorOutput struct {
	Status string          `json:"status"`
	Result json.RawMessage `json:"result"`
}

type GeneratedTest struct {
	TlaFile       string `json:"tla_file"`
	TlaConfigFile string `json:"tla_config_file"`
}

type TlaTrace struct {
	TlaTraceFile string `json:"tla_trace_file"`
}

type JsonTrace struct {
	JsonTraceFile string `json:"json_trace_file"`
}
