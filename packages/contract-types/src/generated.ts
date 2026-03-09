//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// IEAS
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

export const ieasAbi = [
  {
    type: 'function',
    inputs: [
      {
        name: 'request',
        internalType: 'struct AttestationRequest',
        type: 'tuple',
        components: [
          { name: 'schema', internalType: 'bytes32', type: 'bytes32' },
          {
            name: 'data',
            internalType: 'struct AttestationRequestData',
            type: 'tuple',
            components: [
              { name: 'recipient', internalType: 'address', type: 'address' },
              {
                name: 'expirationTime',
                internalType: 'uint64',
                type: 'uint64',
              },
              { name: 'revocable', internalType: 'bool', type: 'bool' },
              { name: 'refUID', internalType: 'bytes32', type: 'bytes32' },
              { name: 'data', internalType: 'bytes', type: 'bytes' },
              { name: 'value', internalType: 'uint256', type: 'uint256' },
            ],
          },
        ],
      },
    ],
    name: 'attest',
    outputs: [{ name: '', internalType: 'bytes32', type: 'bytes32' }],
    stateMutability: 'payable',
  },
  {
    type: 'function',
    inputs: [
      {
        name: 'delegatedRequest',
        internalType: 'struct DelegatedAttestationRequest',
        type: 'tuple',
        components: [
          { name: 'schema', internalType: 'bytes32', type: 'bytes32' },
          {
            name: 'data',
            internalType: 'struct AttestationRequestData',
            type: 'tuple',
            components: [
              { name: 'recipient', internalType: 'address', type: 'address' },
              {
                name: 'expirationTime',
                internalType: 'uint64',
                type: 'uint64',
              },
              { name: 'revocable', internalType: 'bool', type: 'bool' },
              { name: 'refUID', internalType: 'bytes32', type: 'bytes32' },
              { name: 'data', internalType: 'bytes', type: 'bytes' },
              { name: 'value', internalType: 'uint256', type: 'uint256' },
            ],
          },
          {
            name: 'signature',
            internalType: 'struct Signature',
            type: 'tuple',
            components: [
              { name: 'v', internalType: 'uint8', type: 'uint8' },
              { name: 'r', internalType: 'bytes32', type: 'bytes32' },
              { name: 's', internalType: 'bytes32', type: 'bytes32' },
            ],
          },
          { name: 'attester', internalType: 'address', type: 'address' },
          { name: 'deadline', internalType: 'uint64', type: 'uint64' },
        ],
      },
    ],
    name: 'attestByDelegation',
    outputs: [{ name: '', internalType: 'bytes32', type: 'bytes32' }],
    stateMutability: 'payable',
  },
  {
    type: 'function',
    inputs: [{ name: 'uid', internalType: 'bytes32', type: 'bytes32' }],
    name: 'getAttestation',
    outputs: [
      {
        name: '',
        internalType: 'struct Attestation',
        type: 'tuple',
        components: [
          { name: 'uid', internalType: 'bytes32', type: 'bytes32' },
          { name: 'schema', internalType: 'bytes32', type: 'bytes32' },
          { name: 'time', internalType: 'uint64', type: 'uint64' },
          {
            name: 'expirationTime',
            internalType: 'uint64',
            type: 'uint64',
          },
          {
            name: 'revocationTime',
            internalType: 'uint64',
            type: 'uint64',
          },
          { name: 'refUID', internalType: 'bytes32', type: 'bytes32' },
          { name: 'recipient', internalType: 'address', type: 'address' },
          { name: 'attester', internalType: 'address', type: 'address' },
          { name: 'revocable', internalType: 'bool', type: 'bool' },
          { name: 'data', internalType: 'bytes', type: 'bytes' },
        ],
      },
    ],
    stateMutability: 'view',
  },
  {
    type: 'function',
    inputs: [
      { name: 'revoker', internalType: 'address', type: 'address' },
      { name: 'data', internalType: 'bytes32', type: 'bytes32' },
    ],
    name: 'getRevokeOffchain',
    outputs: [{ name: '', internalType: 'uint64', type: 'uint64' }],
    stateMutability: 'view',
  },
  {
    type: 'function',
    inputs: [],
    name: 'getSchemaRegistry',
    outputs: [
      { name: '', internalType: 'contract ISchemaRegistry', type: 'address' },
    ],
    stateMutability: 'view',
  },
  {
    type: 'function',
    inputs: [{ name: 'data', internalType: 'bytes32', type: 'bytes32' }],
    name: 'getTimestamp',
    outputs: [{ name: '', internalType: 'uint64', type: 'uint64' }],
    stateMutability: 'view',
  },
  {
    type: 'function',
    inputs: [{ name: 'uid', internalType: 'bytes32', type: 'bytes32' }],
    name: 'isAttestationValid',
    outputs: [{ name: '', internalType: 'bool', type: 'bool' }],
    stateMutability: 'view',
  },
  {
    type: 'function',
    inputs: [
      {
        name: 'multiRequests',
        internalType: 'struct MultiAttestationRequest[]',
        type: 'tuple[]',
        components: [
          { name: 'schema', internalType: 'bytes32', type: 'bytes32' },
          {
            name: 'data',
            internalType: 'struct AttestationRequestData[]',
            type: 'tuple[]',
            components: [
              { name: 'recipient', internalType: 'address', type: 'address' },
              {
                name: 'expirationTime',
                internalType: 'uint64',
                type: 'uint64',
              },
              { name: 'revocable', internalType: 'bool', type: 'bool' },
              { name: 'refUID', internalType: 'bytes32', type: 'bytes32' },
              { name: 'data', internalType: 'bytes', type: 'bytes' },
              { name: 'value', internalType: 'uint256', type: 'uint256' },
            ],
          },
        ],
      },
    ],
    name: 'multiAttest',
    outputs: [{ name: '', internalType: 'bytes32[]', type: 'bytes32[]' }],
    stateMutability: 'payable',
  },
  {
    type: 'function',
    inputs: [
      {
        name: 'multiDelegatedRequests',
        internalType: 'struct MultiDelegatedAttestationRequest[]',
        type: 'tuple[]',
        components: [
          { name: 'schema', internalType: 'bytes32', type: 'bytes32' },
          {
            name: 'data',
            internalType: 'struct AttestationRequestData[]',
            type: 'tuple[]',
            components: [
              { name: 'recipient', internalType: 'address', type: 'address' },
              {
                name: 'expirationTime',
                internalType: 'uint64',
                type: 'uint64',
              },
              { name: 'revocable', internalType: 'bool', type: 'bool' },
              { name: 'refUID', internalType: 'bytes32', type: 'bytes32' },
              { name: 'data', internalType: 'bytes', type: 'bytes' },
              { name: 'value', internalType: 'uint256', type: 'uint256' },
            ],
          },
          {
            name: 'signatures',
            internalType: 'struct Signature[]',
            type: 'tuple[]',
            components: [
              { name: 'v', internalType: 'uint8', type: 'uint8' },
              { name: 'r', internalType: 'bytes32', type: 'bytes32' },
              { name: 's', internalType: 'bytes32', type: 'bytes32' },
            ],
          },
          { name: 'attester', internalType: 'address', type: 'address' },
          { name: 'deadline', internalType: 'uint64', type: 'uint64' },
        ],
      },
    ],
    name: 'multiAttestByDelegation',
    outputs: [{ name: '', internalType: 'bytes32[]', type: 'bytes32[]' }],
    stateMutability: 'payable',
  },
  {
    type: 'function',
    inputs: [
      {
        name: 'multiRequests',
        internalType: 'struct MultiRevocationRequest[]',
        type: 'tuple[]',
        components: [
          { name: 'schema', internalType: 'bytes32', type: 'bytes32' },
          {
            name: 'data',
            internalType: 'struct RevocationRequestData[]',
            type: 'tuple[]',
            components: [
              { name: 'uid', internalType: 'bytes32', type: 'bytes32' },
              { name: 'value', internalType: 'uint256', type: 'uint256' },
            ],
          },
        ],
      },
    ],
    name: 'multiRevoke',
    outputs: [],
    stateMutability: 'payable',
  },
  {
    type: 'function',
    inputs: [
      {
        name: 'multiDelegatedRequests',
        internalType: 'struct MultiDelegatedRevocationRequest[]',
        type: 'tuple[]',
        components: [
          { name: 'schema', internalType: 'bytes32', type: 'bytes32' },
          {
            name: 'data',
            internalType: 'struct RevocationRequestData[]',
            type: 'tuple[]',
            components: [
              { name: 'uid', internalType: 'bytes32', type: 'bytes32' },
              { name: 'value', internalType: 'uint256', type: 'uint256' },
            ],
          },
          {
            name: 'signatures',
            internalType: 'struct Signature[]',
            type: 'tuple[]',
            components: [
              { name: 'v', internalType: 'uint8', type: 'uint8' },
              { name: 'r', internalType: 'bytes32', type: 'bytes32' },
              { name: 's', internalType: 'bytes32', type: 'bytes32' },
            ],
          },
          { name: 'revoker', internalType: 'address', type: 'address' },
          { name: 'deadline', internalType: 'uint64', type: 'uint64' },
        ],
      },
    ],
    name: 'multiRevokeByDelegation',
    outputs: [],
    stateMutability: 'payable',
  },
  {
    type: 'function',
    inputs: [{ name: 'data', internalType: 'bytes32[]', type: 'bytes32[]' }],
    name: 'multiRevokeOffchain',
    outputs: [{ name: '', internalType: 'uint64', type: 'uint64' }],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    inputs: [{ name: 'data', internalType: 'bytes32[]', type: 'bytes32[]' }],
    name: 'multiTimestamp',
    outputs: [{ name: '', internalType: 'uint64', type: 'uint64' }],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    inputs: [
      {
        name: 'request',
        internalType: 'struct RevocationRequest',
        type: 'tuple',
        components: [
          { name: 'schema', internalType: 'bytes32', type: 'bytes32' },
          {
            name: 'data',
            internalType: 'struct RevocationRequestData',
            type: 'tuple',
            components: [
              { name: 'uid', internalType: 'bytes32', type: 'bytes32' },
              { name: 'value', internalType: 'uint256', type: 'uint256' },
            ],
          },
        ],
      },
    ],
    name: 'revoke',
    outputs: [],
    stateMutability: 'payable',
  },
  {
    type: 'function',
    inputs: [
      {
        name: 'delegatedRequest',
        internalType: 'struct DelegatedRevocationRequest',
        type: 'tuple',
        components: [
          { name: 'schema', internalType: 'bytes32', type: 'bytes32' },
          {
            name: 'data',
            internalType: 'struct RevocationRequestData',
            type: 'tuple',
            components: [
              { name: 'uid', internalType: 'bytes32', type: 'bytes32' },
              { name: 'value', internalType: 'uint256', type: 'uint256' },
            ],
          },
          {
            name: 'signature',
            internalType: 'struct Signature',
            type: 'tuple',
            components: [
              { name: 'v', internalType: 'uint8', type: 'uint8' },
              { name: 'r', internalType: 'bytes32', type: 'bytes32' },
              { name: 's', internalType: 'bytes32', type: 'bytes32' },
            ],
          },
          { name: 'revoker', internalType: 'address', type: 'address' },
          { name: 'deadline', internalType: 'uint64', type: 'uint64' },
        ],
      },
    ],
    name: 'revokeByDelegation',
    outputs: [],
    stateMutability: 'payable',
  },
  {
    type: 'function',
    inputs: [{ name: 'data', internalType: 'bytes32', type: 'bytes32' }],
    name: 'revokeOffchain',
    outputs: [{ name: '', internalType: 'uint64', type: 'uint64' }],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    inputs: [{ name: 'data', internalType: 'bytes32', type: 'bytes32' }],
    name: 'timestamp',
    outputs: [{ name: '', internalType: 'uint64', type: 'uint64' }],
    stateMutability: 'nonpayable',
  },
  {
    type: 'event',
    anonymous: false,
    inputs: [
      {
        name: 'recipient',
        internalType: 'address',
        type: 'address',
        indexed: true,
      },
      {
        name: 'attester',
        internalType: 'address',
        type: 'address',
        indexed: true,
      },
      { name: 'uid', internalType: 'bytes32', type: 'bytes32', indexed: false },
      {
        name: 'schemaUID',
        internalType: 'bytes32',
        type: 'bytes32',
        indexed: true,
      },
    ],
    name: 'Attested',
  },
  {
    type: 'event',
    anonymous: false,
    inputs: [
      {
        name: 'recipient',
        internalType: 'address',
        type: 'address',
        indexed: true,
      },
      {
        name: 'attester',
        internalType: 'address',
        type: 'address',
        indexed: true,
      },
      { name: 'uid', internalType: 'bytes32', type: 'bytes32', indexed: false },
      {
        name: 'schemaUID',
        internalType: 'bytes32',
        type: 'bytes32',
        indexed: true,
      },
    ],
    name: 'Revoked',
  },
  {
    type: 'event',
    anonymous: false,
    inputs: [
      {
        name: 'revoker',
        internalType: 'address',
        type: 'address',
        indexed: true,
      },
      {
        name: 'data',
        internalType: 'bytes32',
        type: 'bytes32',
        indexed: true,
      },
      {
        name: 'timestamp',
        internalType: 'uint64',
        type: 'uint64',
        indexed: true,
      },
    ],
    name: 'RevokedOffchain',
  },
  {
    type: 'event',
    anonymous: false,
    inputs: [
      {
        name: 'data',
        internalType: 'bytes32',
        type: 'bytes32',
        indexed: true,
      },
      {
        name: 'timestamp',
        internalType: 'uint64',
        type: 'uint64',
        indexed: true,
      },
    ],
    name: 'Timestamped',
  },
] as const

//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// IFeeOracle
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

export const iFeeOracleAbi = [
  {
    type: 'function',
    inputs: [],
    name: 'getFloorFee',
    outputs: [{ name: '', internalType: 'uint256', type: 'uint256' }],
    stateMutability: 'view',
  },
  {
    type: 'function',
    inputs: [
      {
        name: 'crossDomainGasEstimated',
        internalType: 'uint256',
        type: 'uint256',
      },
    ],
    name: 'setCrossDomainGasEstimated',
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    inputs: [
      { name: 'expectedBatchSize', internalType: 'uint256', type: 'uint256' },
    ],
    name: 'setExpectedBatchSize',
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    inputs: [
      { name: 'feeMultiplier', internalType: 'uint256', type: 'uint256' },
    ],
    name: 'setFeeMultiplier',
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    inputs: [{ name: 'l1FeeScalar', internalType: 'uint256', type: 'uint256' }],
    name: 'setL1FeeScalar',
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    inputs: [
      { name: 'l1GasEstimated', internalType: 'uint256', type: 'uint256' },
    ],
    name: 'setL1GasEstimated',
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    inputs: [{ name: 'l1Overhead', internalType: 'uint256', type: 'uint256' }],
    name: 'setL1Overhead',
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    inputs: [
      { name: 'l2ExecutionOverhead', internalType: 'uint256', type: 'uint256' },
    ],
    name: 'setL2ExecutionOverhead',
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    inputs: [
      { name: 'l2ExecutionScalar', internalType: 'uint256', type: 'uint256' },
    ],
    name: 'setL2ExecutionScalar',
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'event',
    anonymous: false,
    inputs: [
      {
        name: 'crossDomainGasEstimated',
        internalType: 'uint256',
        type: 'uint256',
        indexed: false,
      },
    ],
    name: 'CrossDomainGasEstimatedUpdated',
  },
  {
    type: 'event',
    anonymous: false,
    inputs: [
      {
        name: 'expectedBatchSize',
        internalType: 'uint256',
        type: 'uint256',
        indexed: false,
      },
    ],
    name: 'ExpectedBatchSizeUpdated',
  },
  {
    type: 'event',
    anonymous: false,
    inputs: [
      {
        name: 'feeMultiplier',
        internalType: 'uint256',
        type: 'uint256',
        indexed: false,
      },
    ],
    name: 'FeeMultiplierUpdated',
  },
  {
    type: 'event',
    anonymous: false,
    inputs: [
      {
        name: 'l1FeeScalar',
        internalType: 'uint256',
        type: 'uint256',
        indexed: false,
      },
    ],
    name: 'L1FeeScalarUpdated',
  },
  {
    type: 'event',
    anonymous: false,
    inputs: [
      {
        name: 'l1GasEstimated',
        internalType: 'uint256',
        type: 'uint256',
        indexed: false,
      },
    ],
    name: 'L1GasEstimatedUpdated',
  },
  {
    type: 'event',
    anonymous: false,
    inputs: [
      {
        name: 'l1Overhead',
        internalType: 'uint256',
        type: 'uint256',
        indexed: false,
      },
    ],
    name: 'L1OverheadUpdated',
  },
  {
    type: 'event',
    anonymous: false,
    inputs: [
      {
        name: 'l2ExecutionOverhead',
        internalType: 'uint256',
        type: 'uint256',
        indexed: false,
      },
    ],
    name: 'L2ExecutionOverheadUpdated',
  },
  {
    type: 'event',
    anonymous: false,
    inputs: [
      {
        name: 'l2ExecutionScalar',
        internalType: 'uint256',
        type: 'uint256',
        indexed: false,
      },
    ],
    name: 'L2ExecutionScalarUpdated',
  },
] as const

//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// IL1AnchoringGateway
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

export const il1AnchoringGatewayAbi = [
  {
    type: 'function',
    inputs: [
      { name: 'newMessenger', internalType: 'address', type: 'address' },
    ],
    name: 'setL1ScrollMessenger',
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    inputs: [{ name: 'newManager', internalType: 'address', type: 'address' }],
    name: 'setL2AnchoringManager',
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    inputs: [
      { name: 'merkleRoot', internalType: 'bytes32', type: 'bytes32' },
      { name: 'startIndex', internalType: 'uint256', type: 'uint256' },
      { name: 'count', internalType: 'uint256', type: 'uint256' },
      { name: 'gasLimit', internalType: 'uint256', type: 'uint256' },
    ],
    name: 'submitBatch',
    outputs: [],
    stateMutability: 'payable',
  },
  {
    type: 'event',
    anonymous: false,
    inputs: [
      {
        name: 'merkleRoot',
        internalType: 'bytes32',
        type: 'bytes32',
        indexed: true,
      },
      {
        name: 'startIndex',
        internalType: 'uint256',
        type: 'uint256',
        indexed: true,
      },
      {
        name: 'count',
        internalType: 'uint256',
        type: 'uint256',
        indexed: false,
      },
      {
        name: 'submitter',
        internalType: 'address',
        type: 'address',
        indexed: true,
      },
    ],
    name: 'BatchSubmitted',
  },
  {
    type: 'event',
    anonymous: false,
    inputs: [
      {
        name: 'oldMessenger',
        internalType: 'address',
        type: 'address',
        indexed: true,
      },
      {
        name: 'newMessenger',
        internalType: 'address',
        type: 'address',
        indexed: true,
      },
    ],
    name: 'L1ScrollMessengerUpdated',
  },
  {
    type: 'event',
    anonymous: false,
    inputs: [
      {
        name: 'oldManager',
        internalType: 'address',
        type: 'address',
        indexed: true,
      },
      {
        name: 'newManager',
        internalType: 'address',
        type: 'address',
        indexed: true,
      },
    ],
    name: 'L2AnchoringManagerUpdated',
  },
] as const

//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// IL2AnchoringManager
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

export const il2AnchoringManagerAbi = [
  {
    type: 'function',
    inputs: [
      { name: 'attestationId', internalType: 'bytes32', type: 'bytes32' },
      { name: 'batchStartIndexHint', internalType: 'uint256', type: 'uint256' },
    ],
    name: 'claimNFT',
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    inputs: [],
    name: 'clearBatch',
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    inputs: [],
    name: 'finalizeBatch',
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    inputs: [
      { name: 'attestationId', internalType: 'bytes32', type: 'bytes32' },
    ],
    name: 'isConfirmed',
    outputs: [{ name: '', internalType: 'bool', type: 'bool' }],
    stateMutability: 'view',
  },
  {
    type: 'function',
    inputs: [
      { name: 'expectedRoot', internalType: 'bytes32', type: 'bytes32' },
      { name: 'startIndex', internalType: 'uint256', type: 'uint256' },
      { name: 'count', internalType: 'uint256', type: 'uint256' },
      { name: 'l1Timestamp', internalType: 'uint256', type: 'uint256' },
      { name: 'l1BlockNumber', internalType: 'uint256', type: 'uint256' },
    ],
    name: 'notifyAnchored',
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    inputs: [{ name: 'oracle', internalType: 'address', type: 'address' }],
    name: 'setFeeOracle',
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    inputs: [{ name: 'l1Gateway', internalType: 'address', type: 'address' }],
    name: 'setL1Gateway',
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    inputs: [{ name: 'l2Messenger', internalType: 'address', type: 'address' }],
    name: 'setL2Messenger',
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    inputs: [
      { name: 'attestationId', internalType: 'bytes32', type: 'bytes32' },
      { name: 'refundAddress', internalType: 'address', type: 'address' },
    ],
    name: 'submitForL1Anchoring',
    outputs: [],
    stateMutability: 'payable',
  },
  {
    type: 'function',
    inputs: [
      { name: 'attestationId', internalType: 'bytes32', type: 'bytes32' },
    ],
    name: 'submitForL1Anchoring',
    outputs: [],
    stateMutability: 'payable',
  },
  {
    type: 'function',
    inputs: [
      { name: 'to', internalType: 'address', type: 'address' },
      { name: 'amount', internalType: 'uint256', type: 'uint256' },
    ],
    name: 'withdrawFees',
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'event',
    anonymous: false,
    inputs: [
      {
        name: 'oldOracle',
        internalType: 'address',
        type: 'address',
        indexed: true,
      },
      {
        name: 'newOracle',
        internalType: 'address',
        type: 'address',
        indexed: true,
      },
    ],
    name: 'FeeOracleUpdated',
  },
  {
    type: 'event',
    anonymous: false,
    inputs: [
      { name: 'to', internalType: 'address', type: 'address', indexed: true },
      {
        name: 'amount',
        internalType: 'uint256',
        type: 'uint256',
        indexed: false,
      },
    ],
    name: 'FeesWithdrawn',
  },
  {
    type: 'event',
    anonymous: false,
    inputs: [
      {
        name: 'attestationId',
        internalType: 'bytes32',
        type: 'bytes32',
        indexed: true,
      },
      { name: 'root', internalType: 'bytes32', type: 'bytes32', indexed: true },
      {
        name: 'queueIndex',
        internalType: 'uint256',
        type: 'uint256',
        indexed: false,
      },
      { name: 'fee', internalType: 'uint256', type: 'uint256', indexed: false },
      {
        name: 'blockNumber',
        internalType: 'uint256',
        type: 'uint256',
        indexed: false,
      },
      {
        name: 'timestamp',
        internalType: 'uint256',
        type: 'uint256',
        indexed: false,
      },
    ],
    name: 'L1AnchoringQueued',
  },
  {
    type: 'event',
    anonymous: false,
    inputs: [
      {
        name: 'claimedRoot',
        internalType: 'bytes32',
        type: 'bytes32',
        indexed: true,
      },
      {
        name: 'startIndex',
        internalType: 'uint256',
        type: 'uint256',
        indexed: true,
      },
      {
        name: 'count',
        internalType: 'uint256',
        type: 'uint256',
        indexed: false,
      },
      {
        name: 'l1BlockAttested',
        internalType: 'uint256',
        type: 'uint256',
        indexed: false,
      },
      {
        name: 'l1TimestampAttested',
        internalType: 'uint256',
        type: 'uint256',
        indexed: false,
      },
      {
        name: 'l2BlockNumber',
        internalType: 'uint256',
        type: 'uint256',
        indexed: false,
      },
      {
        name: 'l2TimestampReceived',
        internalType: 'uint256',
        type: 'uint256',
        indexed: false,
      },
    ],
    name: 'L1BatchArrived',
  },
  {
    type: 'event',
    anonymous: false,
    inputs: [
      {
        name: 'merkleRoot',
        internalType: 'bytes32',
        type: 'bytes32',
        indexed: true,
      },
      {
        name: 'startIndex',
        internalType: 'uint256',
        type: 'uint256',
        indexed: true,
      },
      {
        name: 'count',
        internalType: 'uint256',
        type: 'uint256',
        indexed: false,
      },
      {
        name: 'l1BlockAttested',
        internalType: 'uint256',
        type: 'uint256',
        indexed: false,
      },
      {
        name: 'l1TimestampAttested',
        internalType: 'uint256',
        type: 'uint256',
        indexed: false,
      },
      {
        name: 'l2BlockNumber',
        internalType: 'uint256',
        type: 'uint256',
        indexed: false,
      },
      {
        name: 'l2TimestampFinalized',
        internalType: 'uint256',
        type: 'uint256',
        indexed: false,
      },
    ],
    name: 'L1BatchFinalized',
  },
  {
    type: 'event',
    anonymous: false,
    inputs: [
      {
        name: 'oldGateway',
        internalType: 'address',
        type: 'address',
        indexed: true,
      },
      {
        name: 'newGateway',
        internalType: 'address',
        type: 'address',
        indexed: true,
      },
    ],
    name: 'L1GatewayUpdated',
  },
  {
    type: 'event',
    anonymous: false,
    inputs: [
      {
        name: 'oldMessenger',
        internalType: 'address',
        type: 'address',
        indexed: true,
      },
      {
        name: 'newMessenger',
        internalType: 'address',
        type: 'address',
        indexed: true,
      },
    ],
    name: 'L2MessengerUpdated',
  },
  {
    type: 'event',
    anonymous: false,
    inputs: [
      {
        name: 'submitter',
        internalType: 'address',
        type: 'address',
        indexed: true,
      },
      {
        name: 'tokenId',
        internalType: 'uint256',
        type: 'uint256',
        indexed: true,
      },
      { name: 'root', internalType: 'bytes32', type: 'bytes32', indexed: true },
      {
        name: 'timestamp',
        internalType: 'uint256',
        type: 'uint256',
        indexed: false,
      },
    ],
    name: 'NFTClaimed',
  },
  {
    type: 'event',
    anonymous: false,
    inputs: [
      {
        name: 'oldUts',
        internalType: 'address',
        type: 'address',
        indexed: true,
      },
      {
        name: 'newUts',
        internalType: 'address',
        type: 'address',
        indexed: true,
      },
    ],
    name: 'UtsUpdated',
  },
] as const
