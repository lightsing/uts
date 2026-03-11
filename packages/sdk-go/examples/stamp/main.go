package main

import (
	"context"
	"encoding/hex"
	"fmt"
	"log"
	"os"
	"time"

	uts "github.com/uts-dot/sdk-go"
	"github.com/uts-dot/sdk-go/codec"
	"github.com/uts-dot/sdk-go/crypto"
	"github.com/uts-dot/sdk-go/types"
)

func main() {
	if len(os.Args) < 2 {
		fmt.Println("Usage: stamp <file> [output.uts]")
		fmt.Println("       stamp --sample [output.uts]")
		os.Exit(1)
	}

	var data []byte
	var outputFile string
	var useSample bool

	if os.Args[1] == "--sample" {
		useSample = true
		data = []byte("Hello, Universal Timestamps!")
		if len(os.Args) > 2 {
			outputFile = os.Args[2]
		} else {
			outputFile = "sample.uts"
		}
	} else {
		inputFile := os.Args[1]
		if len(os.Args) > 2 {
			outputFile = os.Args[2]
		} else {
			outputFile = inputFile + ".uts"
		}

		var err error
		data, err = os.ReadFile(inputFile)
		if err != nil {
			log.Fatalf("Failed to read file: %v", err)
		}
	}

	hash := crypto.Keccak256(data)
	if useSample {
		fmt.Printf("Sample data: %q\n", string(data))
	} else {
		fmt.Printf("File: %s (%d bytes)\n", os.Args[1], len(data))
	}
	fmt.Printf("Hash (keccak256): %s\n", hex.EncodeToString(hash[:]))

	sdk := uts.NewSDK(
		uts.WithCalendars("https://lgm1.test.timestamps.now/"),
		uts.WithTimeout(30*time.Second),
	)

	header := types.NewDigestHeader(types.DigestKECCAK256, hash[:])

	ctx, cancel := context.WithTimeout(context.Background(), 60*time.Second)
	defer cancel()

	stamps, err := sdk.Stamp(ctx, []*types.DigestHeader{header})
	if err != nil {
		log.Fatalf("Failed to stamp: %v", err)
	}

	if len(stamps) == 0 {
		log.Fatal("No stamps returned")
	}

	stamp := stamps[0]
	fmt.Printf("\nTimestamp created with %d steps\n", len(stamp.Timestamp))
	for i, step := range stamp.Timestamp {
		fmt.Printf("  %d: %s\n", i+1, step)
	}

	encoded, err := codec.EncodeDetachedTimestamp(stamp)
	if err != nil {
		log.Fatalf("Failed to encode timestamp: %v", err)
	}

	if err := os.WriteFile(outputFile, encoded, 0644); err != nil {
		log.Fatalf("Failed to write output file: %v", err)
	}

	fmt.Printf("\nTimestamp saved to: %s (%d bytes)\n", outputFile, len(encoded))
}
