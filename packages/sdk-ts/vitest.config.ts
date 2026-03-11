import { defineConfig } from 'vitest/config'
import path from 'node:path'

export default defineConfig({
  resolve: {
    alias: {
      '@uts/contracts': path.resolve(
        __dirname,
        '../contract-types/src/index.ts',
      ),
    },
  },
  test: {
    environment: 'node',

    include: ['**/*.{test,spec}.{js,mjs,cjs,ts,mts,cts,jsx,tsx}'],

    testTimeout: 10000,
  },
})
