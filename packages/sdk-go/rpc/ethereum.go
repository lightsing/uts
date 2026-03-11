package rpc

import (
	"context"
	"fmt"
	"math/big"
	"strings"
	"sync"

	"github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/ethclient"
)

var (
	SchemaID = [32]byte{
		0x5c, 0x5b, 0x8b, 0x29, 0x5f, 0xf4, 0x3c, 0x8e,
		0x44, 0x2b, 0xe1, 0x1d, 0x56, 0x9e, 0x94, 0xa4,
		0xcd, 0x54, 0x76, 0xf5, 0xe2, 0x3d, 0xf0, 0xf7,
		0x1b, 0xdd, 0x40, 0x8d, 0xf6, 0xb9, 0x64, 0x9c,
	}

	EASAddresses = map[uint64]common.Address{
		1:        common.HexToAddress("0xA1207F3BBa224E2c9c3c6D5aF63D0eb1582Ce587"),
		534351:   common.HexToAddress("0xaEF4103A04090071165F78D45D83A0C0782c2B2a"),
		534352:   common.HexToAddress("0xC47300428b6AD2c7D03BB76D05A176058b47E6B0"),
		11155111: common.HexToAddress("0xC47300428b6AD2c7D03BB76D05A176058b47E6B0"),
	}
)

type Attestation struct {
	UID            [32]byte
	Schema         [32]byte
	Time           uint64
	ExpirationTime uint64
	RevocationTime uint64
	RefUID         [32]byte
	Recipient      common.Address
	Attester       common.Address
	Revocable      bool
	Data           []byte
}

type EthereumClient struct {
	mu      sync.RWMutex
	clients map[uint64]*ethclient.Client
}

func NewEthereumClient() *EthereumClient {
	return &EthereumClient{
		clients: make(map[uint64]*ethclient.Client),
	}
}

func (c *EthereumClient) AddChain(chainID uint64, rpcURL string) error {
	client, err := ethclient.Dial(rpcURL)
	if err != nil {
		return fmt.Errorf("failed to connect to RPC: %w", err)
	}

	c.mu.Lock()
	defer c.mu.Unlock()
	c.clients[chainID] = client
	return nil
}

func (c *EthereumClient) GetClient(chainID uint64) (*ethclient.Client, bool) {
	c.mu.RLock()
	defer c.mu.RUnlock()
	client, ok := c.clients[chainID]
	return client, ok
}

func (c *EthereumClient) GetEASAttestation(ctx context.Context, chainID uint64, uid [32]byte) (*Attestation, error) {
	client, ok := c.GetClient(chainID)
	if !ok {
		return nil, fmt.Errorf("no client configured for chain %d", chainID)
	}

	addr, ok := EASAddresses[chainID]
	if !ok {
		return nil, fmt.Errorf("no EAS address configured for chain %d", chainID)
	}

	parsedABI, err := EASABI()
	if err != nil {
		return nil, fmt.Errorf("failed to parse ABI: %w", err)
	}

	callData, err := parsedABI.Pack("getAttestation", uid)
	if err != nil {
		return nil, fmt.Errorf("failed to pack call data: %w", err)
	}

	result, err := client.CallContract(ctx, ethereum.CallMsg{
		To:   &addr,
		Data: callData,
	}, nil)
	if err != nil {
		return nil, fmt.Errorf("contract call failed: %w", err)
	}

	var att Attestation
	var rawAtt struct {
		UID            [32]byte
		Schema         [32]byte
		Time           *big.Int
		ExpirationTime *big.Int
		RevocationTime *big.Int
		RefUID         [32]byte
		Recipient      common.Address
		Attester       common.Address
		Revocable      bool
		Data           []byte
	}

	err = parsedABI.UnpackIntoInterface(&rawAtt, "getAttestation", result)
	if err != nil {
		return nil, fmt.Errorf("failed to unpack result: %w", err)
	}

	att.UID = rawAtt.UID
	att.Schema = rawAtt.Schema
	att.Time = rawAtt.Time.Uint64()
	att.ExpirationTime = rawAtt.ExpirationTime.Uint64()
	att.RevocationTime = rawAtt.RevocationTime.Uint64()
	att.RefUID = rawAtt.RefUID
	att.Recipient = rawAtt.Recipient
	att.Attester = rawAtt.Attester
	att.Revocable = rawAtt.Revocable
	att.Data = rawAtt.Data

	return &att, nil
}

func (c *EthereumClient) GetTimestamp(ctx context.Context, chainID uint64, data [32]byte) (uint64, error) {
	client, ok := c.GetClient(chainID)
	if !ok {
		return 0, fmt.Errorf("no client configured for chain %d", chainID)
	}

	addr, ok := EASAddresses[chainID]
	if !ok {
		return 0, fmt.Errorf("no EAS address configured for chain %d", chainID)
	}

	parsedABI, err := EASABI()
	if err != nil {
		return 0, fmt.Errorf("failed to parse ABI: %w", err)
	}

	callData, err := parsedABI.Pack("getTimestamp", data)
	if err != nil {
		return 0, fmt.Errorf("failed to pack call data: %w", err)
	}

	result, err := client.CallContract(ctx, ethereum.CallMsg{
		To:   &addr,
		Data: callData,
	}, nil)
	if err != nil {
		return 0, fmt.Errorf("contract call failed: %w", err)
	}

	var timestamp *big.Int
	err = parsedABI.UnpackIntoInterface(&timestamp, "getTimestamp", result)
	if err != nil {
		return 0, fmt.Errorf("failed to unpack result: %w", err)
	}

	return timestamp.Uint64(), nil
}

func (c *EthereumClient) Close() {
	c.mu.Lock()
	defer c.mu.Unlock()
	for _, client := range c.clients {
		client.Close()
	}
	c.clients = make(map[uint64]*ethclient.Client)
}

func (c *EthereumClient) GetBlockNumber(ctx context.Context, chainID uint64) (uint64, error) {
	client, ok := c.GetClient(chainID)
	if !ok {
		return 0, fmt.Errorf("no client configured for chain %d", chainID)
	}

	header, err := client.HeaderByNumber(ctx, nil)
	if err != nil {
		return 0, fmt.Errorf("failed to get block number: %w", err)
	}
	return header.Number.Uint64(), nil
}

func (c *EthereumClient) GetTransactionReceipt(ctx context.Context, chainID uint64, txHash common.Hash) (*types.Receipt, error) {
	client, ok := c.GetClient(chainID)
	if !ok {
		return nil, fmt.Errorf("no client configured for chain %d", chainID)
	}

	return client.TransactionReceipt(ctx, txHash)
}

type EASContract struct {
	*bind.BoundContract
	address common.Address
	client  *ethclient.Client
}

func NewEASContract(client *ethclient.Client, address common.Address) (*EASContract, error) {
	parsedABI, err := EASABI()
	if err != nil {
		return nil, fmt.Errorf("failed to parse ABI: %w", err)
	}
	return &EASContract{
		BoundContract: bind.NewBoundContract(address, parsedABI, client, nil, nil),
		address:       address,
		client:        client,
	}, nil
}

func EASABI() (abi.ABI, error) {
	const easABI = `[{"inputs":[{"internalType":"bytes32","name":"uid","type":"bytes32"}],"name":"getAttestation","outputs":[{"components":[{"internalType":"bytes32","name":"uid","type":"bytes32"},{"internalType":"bytes32","name":"schema","type":"bytes32"},{"internalType":"uint64","name":"time","type":"uint64"},{"internalType":"uint64","name":"expirationTime","type":"uint64"},{"internalType":"uint64","name":"revocationTime","type":"uint64"},{"internalType":"bytes32","name":"refUID","type":"bytes32"},{"internalType":"address","name":"recipient","type":"address"},{"internalType":"address","name":"attester","type":"address"},{"internalType":"bool","name":"revocable","type":"bool"},{"internalType":"bytes","name":"data","type":"bytes"}],"internalType":"struct Attestation","name":"","type":"tuple"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"bytes32","name":"data","type":"bytes32"}],"name":"getTimestamp","outputs":[{"internalType":"uint64","name":"","type":"uint64"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"bytes32","name":"data","type":"bytes32"}],"name":"timestamp","outputs":[{"internalType":"uint64","name":"","type":"uint64"}],"stateMutability":"nonpayable","type":"function"}]`

	return abi.JSON(strings.NewReader(easABI))
}
