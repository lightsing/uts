import { defineConfig } from '@wagmi/cli'
import { foundry } from '@wagmi/cli/plugins'
import type { Config } from '@wagmi/cli'

export default defineConfig({
  out: 'src/generated.ts',
  contracts: [],
  plugins: [
    foundry({
      project: '../..',
      include: [
        'IL1AnchoringGateway.sol/*.json',
        'IFeeOracle.sol/*.json',
        'IL2AnchoringManager.sol/*.json',
        'IEAS.sol/*.json',
      ],
    }),
  ],
}) as Config
