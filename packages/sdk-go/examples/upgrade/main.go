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
		fmt.Println("Usage: upgrade <timestamp.ots> [output.ots]")
		fmt.Println("\nUpgrades pending attestations in a timestamp by fetching")
		fmt.Println("completed attestations from calendar servers.")
		os.Exit(1)
	}

	inputFile := os.Args[1]
	var outputFile string
	if len(os.Args) > 2 {
		outputFile = os.Args[2]
	} else {
		outputFile = inputFile
	}

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
	fmt.Printf("Steps: %d\n\n", len(stamp.Timestamp))

	pendingCount := countPendingAttestations(stamp.Timestamp)
	fmt.Printf("Pending attestations found: %d\n", pendingCount)

	if pendingCount == 0 {
		fmt.Println("No pending attestations to upgrade.")
		return
	}

	sdk, err := uts.NewSDK(
		uts.WithTimeout(30 * time.Second),
	)
	if err != nil {
		log.Fatalf("Failed to create SDK: %v", err)
	}

	ctx, cancel := context.WithTimeout(context.Background(), 60*time.Second)
	defer cancel()

	results, err := sdk.Upgrade(ctx, stamp, true)
	if err != nil {
		log.Fatalf("Failed to upgrade: %v", err)
	}

	fmt.Printf("\nUpgrade Results:\n")
	upgraded := 0
	stillPending := 0
	failed := 0

	for _, result := range results {
		switch result.Status {
		case types.UpgradeUpgraded:
			upgraded++
			fmt.Printf("  - Upgraded successfully\n")
		case types.UpgradePending:
			stillPending++
			fmt.Printf("  - Still pending (not yet available)\n")
		case types.UpgradeFailed:
			failed++
			fmt.Printf("  - Failed: %v\n", result.Error)
		}
	}

	fmt.Printf("\nSummary: %d upgraded, %d pending, %d failed\n", upgraded, stillPending, failed)

	encoded, err := codec.EncodeDetachedTimestamp(stamp)
	if err != nil {
		log.Fatalf("Failed to encode timestamp: %v", err)
	}

	if err := os.WriteFile(outputFile, encoded, 0644); err != nil {
		log.Fatalf("Failed to write output file: %v", err)
	}

	fmt.Printf("Updated timestamp saved to: %s\n", outputFile)
}

func countPendingAttestations(ts types.Timestamp) int {
	count := 0
	for _, step := range ts {
		switch s := step.(type) {
		case *types.AttestationStep:
			if _, ok := s.Attestation.(*types.PendingAttestation); ok {
				count++
			}
		case *types.ForkStep:
			for _, branch := range s.Branches {
				count += countPendingAttestations(branch)
			}
		}
	}
	return count
}
