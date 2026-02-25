import resolve from '@rollup/plugin-node-resolve'
import typescript from '@rollup/plugin-typescript'
import dts from 'rollup-plugin-dts'

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
    external: [/node_modules/, '@noble/hashes', 'ethers'],
    plugins: [
      resolve(),
      typescript({
        tsconfig: './tsconfig.json',
        declaration: false,
        declarationMap: false,
      }),
    ],
  },
  {
    input: 'src/index.ts',
    output: {
      file: 'dist/index.d.ts',
      format: 'es',
    },
    external: [/node_modules/, '@noble/hashes', 'ethers'],
    plugins: [dts({ tsconfig: './tsconfig.json' })],
  },
]
