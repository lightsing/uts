import { RemoteError } from '../errors'

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
    const response = await fetch(this.url.toString(), {
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
    const data: BitcoinRPCResponse = await response.json()
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
