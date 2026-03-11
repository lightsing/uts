package main

import (
	"context"
	"fmt"
	"log"
	"os"
	"time"

	uts "github.com/lightsing/uts/packages/sdk-go"
	"github.com/lightsing/uts/packages/sdk-go/codec"
	"github.com/lightsing/uts/packages/sdk-go/types"
)

func main() {
	if len(os.Args) < 2 {
		fmt.Println("Usage: verify <timestamp.uts>")
		os.Exit(1)
	}

	inputFile := os.Args[1]

	data, err := os.ReadFile(inputFile)
	if err != nil {
		log.Fatalf("Failed to read timestamp file: %v", err)
	}

	stamp, err := codec.DecodeDetachedTimestamp(data)
	if err != nil {
		log.Fatalf("Failed to decode timestamp: %v", err)
	}

	fmt.Printf("Loaded timestamp from: %s\n", inputFile)
	fmt.Printf("Digest: %s\n", stamp.Header)
	fmt.Printf("Steps: %d\n", len(stamp.Timestamp))

	sdk := uts.NewSDK(
		uts.WithTimeout(30 * time.Second),
	)

	ctx, cancel := context.WithTimeout(context.Background(), 60*time.Second)
	defer cancel()

	result, err := sdk.Verify(ctx, stamp)
	if err != nil {
		log.Fatalf("Failed to verify: %v", err)
	}

	fmt.Printf("\nVerification Result: %s\n", result.Status)

	if len(result.Attestations) > 0 {
		fmt.Printf("\nAttestations (%d):\n", len(result.Attestations))
		for i, att := range result.Attestations {
			fmt.Printf("  %d. %s\n", i+1, att)
		}
	}

	switch result.Status {
	case types.VerifyValid:
		fmt.Println("\nTimestamp is valid and confirmed on-chain.")
		os.Exit(0)
	case types.VerifyPartialValid:
		fmt.Println("\nTimestamp has partial confirmation (some attestations valid).")
		os.Exit(0)
	case types.VerifyPending:
		fmt.Println("\nTimestamp is pending confirmation.")
		os.Exit(0)
	case types.VerifyInvalid:
		fmt.Println("\nTimestamp verification failed.")
		os.Exit(1)
	}
}
