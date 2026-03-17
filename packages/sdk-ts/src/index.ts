// MIT License
//
// Copyright (c) 2025 UTS Contributors
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
//
// Apache License, Version 2.0
//
// Copyright (c) 2025 UTS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import { toHex } from 'viem'

export type * from './types.ts'

export type { BytesLike } from './utils.ts'

export type {
  Attestation,
  PendingAttestation,
  BitcoinAttestation,
  EASAttestation,
  EASTimestamped,
  EthereumUTSAttestationExtraMetadata,
} from './types.ts'

export {
  DIGEST_OPS,
  UpgradeStatus,
  AttestationStatusKind,
  VerifyStatus,
} from './types.ts'

export { default as Encoder } from './codec/encode.ts'
export { default as Decoder } from './codec/decode.ts'

export * from './errors.ts'

export * from './codec/constants.ts'

export * from './bmt.ts'

export { default as BitcoinRPC } from './rpc/btc.ts'

export { default as SDK, DEFAULT_CALENDARS, WELL_KNOWN_CHAINS } from './sdk.ts'
export type {
  EIP1193Provider,
  SDKOptions,
  StampEvent,
  StampEventCallback,
} from './sdk.ts'

export const hexlify = (obj: any): any => {
  if (obj instanceof URL) {
    return obj
  }
  if (obj instanceof Uint8Array) {
    return toHex(obj)
  }
  if (Array.isArray(obj)) {
    return obj.map((item) => hexlify(item))
  }
  if (typeof obj === 'object' && obj !== null) {
    const result: any = {}
    for (const key in obj) {
      if (Object.hasOwn(obj, key)) {
        result[key] = hexlify(obj[key])
      }
    }
    return result
  }
  return obj
}
