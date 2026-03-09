import resolve from '@rollup/plugin-node-resolve'
import typescript from '@rollup/plugin-typescript'

/** @type {import('rollup').RollupOptions[]} */
export default [
  {
    input: 'src/index.ts',
    output: [
      {
        file: 'dist/index.js',
        format: 'es',
        sourcemap: true,
      },
    ],
    external: [/node_modules/, '@noble/hashes', 'viem', '@uts/contracts'],
    plugins: [
      resolve(),
      typescript({
        tsconfig: './tsconfig.json',
        declaration: false,
        declarationMap: false,
      }),
    ],
  },
]
