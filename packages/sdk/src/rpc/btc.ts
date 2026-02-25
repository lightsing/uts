import { RemoteError } from '../errors.ts'

export interface BitcoinRPCResponse {
  jsonrpc: string
  id: number
  result?: any
  error?: {
    code: number
    message: string
    data?: any
  }
}

export interface BitcoinBlockHeader {
  hash: string
  confirmations: number
  height: number
  version: number
  versionHex: string
  merkleroot: string
  time: number
  mediantime: number
  nonce: number
  bits: string
  target: string
  difficulty: number
  chainwork: string
  nTx: number
  previousblockhash?: string
  nextblockhash?: string
}

export default class BitcoinRPC {
  readonly url: URL = new URL('https://bitcoin-rpc.publicnode.com')

  constructor(url?: URL) {
    if (url) {
      this.url = url
    }
  }

  async call(method: string, params: any[] = []): Promise<any> {
    let response: Response
    try {
      response = await fetch(this.url.toString(), {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          jsonrpc: '1.0',
          method,
          params,
          id: 1,
        }),
      })
    } catch (err) {
      throw new RemoteError('Bitcoin RPC network error', {
        context: {
          url: this.url.toString(),
          error: err instanceof Error ? err.message : String(err),
        },
      })
    }

    let rawBody: string
    try {
      rawBody = await response.text()
    } catch (err) {
      throw new RemoteError('Bitcoin RPC error reading response body', {
        context: {
          status: response.status,
          statusText: response.statusText,
          error: err instanceof Error ? err.message : String(err),
        },
      })
    }

    if (!response.ok) {
      throw new RemoteError(
        `Bitcoin RPC HTTP error: ${response.status} ${response.statusText}`,
        {
          context: {
            status: response.status,
            statusText: response.statusText,
            response: rawBody,
          },
        },
      )
    }

    let data: BitcoinRPCResponse
    try {
      data = JSON.parse(rawBody) as BitcoinRPCResponse
    } catch (err) {
      throw new RemoteError('Bitcoin RPC invalid JSON response', {
        context: {
          status: response.status,
          statusText: response.statusText,
          body: rawBody,
          error: err instanceof Error ? err.message : String(err),
        },
      })
    }

    if (data.result !== undefined) {
      return data.result
    }
    if (data.error !== undefined && data.error !== null) {
      throw new RemoteError(`Bitcoin RPC error: ${data.error.message}`, {
        context: {
          code: data.error.code,
          data: data.error.data,
        },
      })
    }
    return data.result
  }

  getBlockHash(height: number): Promise<string> {
    return this.call('getblockhash', [height])
  }

  getBlockHeader(hash: string): Promise<BitcoinBlockHeader> {
    return this.call('getblockheader', [hash])
  }
}
