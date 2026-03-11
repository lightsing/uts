package rpc

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"
)

type BitcoinRPC struct {
	url        string
	httpClient *http.Client
}

type BlockHeader struct {
	Hash          string `json:"hash"`
	MerkleRoot    string `json:"merkleroot"`
	Time          int64  `json:"time"`
	Height        int64  `json:"height,omitempty"`
	Confirmations int64  `json:"confirmations,omitempty"`
}

type jsonRPCRequest struct {
	Jsonrpc string        `json:"jsonrpc"`
	Method  string        `json:"method"`
	Params  []interface{} `json:"params"`
	ID      int           `json:"id"`
}

type jsonRPCResponse struct {
	Jsonrpc string          `json:"jsonrpc"`
	Result  json.RawMessage `json:"result"`
	Error   *jsonRPCError   `json:"error"`
	ID      int             `json:"id"`
}

type jsonRPCError struct {
	Code    int    `json:"code"`
	Message string `json:"message"`
}

func NewBitcoinRPC(url string) *BitcoinRPC {
	return &BitcoinRPC{
		url: url,
		httpClient: &http.Client{
			Timeout: 30 * time.Second,
		},
	}
}

func (c *BitcoinRPC) GetBlockHash(height int64) (string, error) {
	var result string
	if err := c.call("getblockhash", []interface{}{height}, &result); err != nil {
		return "", fmt.Errorf("getblockhash: %w", err)
	}
	return result, nil
}

func (c *BitcoinRPC) GetBlockHeader(hash string) (*BlockHeader, error) {
	var result BlockHeader
	if err := c.call("getblockheader", []interface{}{hash}, &result); err != nil {
		return nil, fmt.Errorf("getblockheader: %w", err)
	}
	return &result, nil
}

func (c *BitcoinRPC) call(method string, params []interface{}, result interface{}) error {
	req := jsonRPCRequest{
		Jsonrpc: "2.0",
		Method:  method,
		Params:  params,
		ID:      1,
	}

	body, err := json.Marshal(req)
	if err != nil {
		return fmt.Errorf("marshal request: %w", err)
	}

	httpReq, err := http.NewRequest("POST", c.url, bytes.NewReader(body))
	if err != nil {
		return fmt.Errorf("create request: %w", err)
	}
	httpReq.Header.Set("Content-Type", "application/json")

	resp, err := c.httpClient.Do(httpReq)
	if err != nil {
		return fmt.Errorf("http request: %w", err)
	}
	defer resp.Body.Close()

	respBody, err := io.ReadAll(resp.Body)
	if err != nil {
		return fmt.Errorf("read response: %w", err)
	}

	var rpcResp jsonRPCResponse
	if err := json.Unmarshal(respBody, &rpcResp); err != nil {
		return fmt.Errorf("unmarshal response: %w", err)
	}

	if rpcResp.Error != nil {
		return fmt.Errorf("rpc error %d: %s", rpcResp.Error.Code, rpcResp.Error.Message)
	}

	if err := json.Unmarshal(rpcResp.Result, result); err != nil {
		return fmt.Errorf("unmarshal result: %w", err)
	}

	return nil
}

func ReverseHexBytes(hexStr string) ([]byte, error) {
	data, err := hex.DecodeString(hexStr)
	if err != nil {
		return nil, fmt.Errorf("decode hex: %w", err)
	}

	for i, j := 0, len(data)-1; i < j; i, j = i+1, j-1 {
		data[i], data[j] = data[j], data[i]
	}
	return data, nil
}

func ReverseBytes(data []byte) []byte {
	result := make([]byte, len(data))
	for i, b := range data {
		result[len(data)-1-i] = b
	}
	return result
}
