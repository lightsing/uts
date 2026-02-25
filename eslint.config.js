// cSpell:disable
// @ts-check
import tseslint from 'typescript-eslint'
import UnicornPlugin from 'eslint-plugin-unicorn'
import UnusedImportsPlugin from 'eslint-plugin-unused-imports'
import { defineConfig } from 'eslint/config'

// Prefer rules from @typescript-eslint > unicorn > other plugins
// Level: if the rule is fixable and can be tolerate during dev, use 'warn' is better.
//        if the fix needs big rewrite (e.g. XHR => fetch), use 'error' to notice the developer early.
//        for RegEx rules, always uses 'error'.

const avoidMistakeRules = {
  // Code quality
  'no-invalid-regexp': 'error', // RegEx
  'unicorn/no-abusive-eslint-disable': 'error', // disable a rule requires a reason
  '@typescript-eslint/ban-ts-comment': [
    'error',
    {
      'ts-expect-error': 'allow-with-description',
      'ts-ignore': true,
      'ts-nocheck': true,
      'ts-check': false,
      minimumDescriptionLength: 5,
    },
  ], // disable a rule requires a reason
  /// TypeScript bad practice
  '@typescript-eslint/no-empty-object-type': [
    'error',
    { allowInterfaces: 'with-single-extends' },
  ],
  // '@typescript-eslint/no-invalid-void-type': 'warn', // Disallow void type outside of generic or return types
  '@typescript-eslint/no-misused-new': 'error', // wrong 'new ()' or 'constructor()' signatures
  '@typescript-eslint/no-unsafe-function-type': 'error',
  // '@typescript-eslint/no-unsafe-type-assertion': 'error', // bans `expr as T`
  '@typescript-eslint/no-wrapper-object-types': 'error',
  /// Unicode support
  'no-misleading-character-class': 'error', // RegEx
  'unicorn/prefer-code-point': 'error',
  /// type safety
  // '@typescript-eslint/method-signature-style': 'warn', // method signature is bivariant
  '@typescript-eslint/no-non-null-asserted-optional-chain': 'error', // bans foo?.bar!
  // '@typescript-eslint/no-unsafe-argument': 'error', // bans call(any)
  // '@typescript-eslint/no-unsafe-assignment': 'error', // bans a = any
  // '@typescript-eslint/no-unsafe-call': 'error', // bans any()
  // '@typescript-eslint/no-unsafe-member-access': 'error', // bans a = any.prop
  // '@typescript-eslint/no-unsafe-return': 'error', // bans return any
  '@typescript-eslint/prefer-return-this-type': 'error', // use `: this` properly
  // '@typescript-eslint/restrict-plus-operands': 'error', // stronger `a + b` check
  // '@typescript-eslint/restrict-template-expressions': 'error', // bans `${nonString}`
  // '@typescript-eslint/strict-boolean-expressions': 'error', // stronger check for nullable string/number/boolean
  // '@typescript-eslint/switch-exhaustiveness-check': 'error', // switch should be exhaustive
  // '@typescript-eslint/unbound-method': 'error', // requires `this` to be set properly

  // Security
  'no-script-url': 'error', // javascript:
  // 'unicorn/require-post-message-target-origin': 'warn', // postMessage(data, 'origin')
  '@typescript-eslint/no-implied-eval': 'error', // setTimeout('code')

  // Confusing code
  'no-constant-binary-expression': 'error', // a + b ?? c
  'no-control-regex': 'error', // RegEx
  'no-div-regex': 'error', // RegEx
  'no-label-var': 'warn', // name collision
  'no-sequences': 'warn', // (a, b)
  '@typescript-eslint/no-confusing-non-null-assertion': 'error', // a! == b

  // Problematic language features
  /// API with trap
  radix: 'warn', // parseInt('1', _required_)
  'unicorn/no-instanceof-builtins': 'warn', // bans `expr instanceof String` etc
  'unicorn/require-array-join-separator': 'warn', // Array.join(_required_)
  // This rule breaks BigNumber class which has different .toFixed() default value.
  // 'unicorn/require-number-to-fixed-digits-argument': 'warn', // Number#toFixed(_required_)
  '@typescript-eslint/require-array-sort-compare': 'error', // Array#sort(_required_)
  /// Footgun language features
  'no-compare-neg-zero': 'error', // x === -0 is wrong
  'no-new-wrappers': 'error', // wrapper objects are bad
  'no-unsafe-finally': 'error', // finally { return expr }
  'unicorn/no-thenable': 'error', // export function then()
  'no-loss-of-precision': 'error', // 5123000000000000000000000000001 is 5123000000000000000000000000000 actually
  '@typescript-eslint/prefer-enum-initializers': 'warn', // add a new item in the middle is an API breaking change.
  /// Little-known language features
  'no-constructor-return': 'error', // constructor() { return expr }
  '@typescript-eslint/no-unsafe-declaration-merging': 'error',
  '@typescript-eslint/no-mixed-enums': 'error', // enum { a = 1, b = "b" }
  '@typescript-eslint/prefer-literal-enum-member': 'error', // enum { a = outsideVar }

  // Prevent bugs
  // 'array-callback-return': 'error', // .map .some ... calls should have a return value
  'default-case-last': 'error', // default: should be the last
  eqeqeq: 'error', // ===
  'no-cond-assign': 'error', // if (a = b)
  'no-duplicate-case': 'error', // switch
  'no-empty-character-class': 'error', // RegEx /[]/ means a empty character class, not "[]"
  'no-global-assign': 'error', // onmessage = ...
  'no-self-assign': 'error', // a = a
  'no-self-compare': 'error', // a === a
  'no-sparse-arrays': 'error', // [,, 1]
  'no-unmodified-loop-condition': 'error', // loop bug
  'no-unreachable-loop': 'error', // loop bug
  'no-restricted-globals': [
    'error',
    // source of bug (those names are too common)
    'error',
    'event',
    'name',
    'length',
    'closed',
    // no localStorage & sessionStorage in a web extension
    {
      name: 'localStorage',
      message:
        "If you're in the background script, localStorage is banned. It will cause Manifest V3 to crash. If you're in the chrome-extension:// pages, localStorage is discouraged. If you're in the content scripts, we can only use localStorage to read websites' data and MUST NOT store our own data.",
    },
    {
      name: 'sessionStorage',
      message:
        "If you're in the background script, sessionStorage is banned. It will cause Manifest V3 to crash. If you're in the chrome-extension:// pages, sessionStorage is discouraged. If you're in the content scripts, we can only use sessionStorage to read websites' data and MUST NOT store our own data.",
    },
  ],
  'no-template-curly-in-string': 'error', // "${expr}" looks like a bug
  // 'require-atomic-updates': 'error', // await/yield race condition
  'valid-typeof': 'error', // typeof expr === undefined
  'unicorn/no-invalid-remove-event-listener': 'error', // removeEventListener('click', f.bind(...))
  'unicorn/no-negation-in-equality-check': 'error', // !foo === bar
  '@typescript-eslint/no-base-to-string': 'error', // prevent buggy .toString() call
  '@typescript-eslint/no-loop-func': 'warn', // capture a loop variable might be a bug
  '@typescript-eslint/no-duplicate-enum-values': 'error', // enum { a = 1, b = 1 }

  // Performance
  'unicorn/consistent-function-scoping': 'warn', // hoist unnecessary higher order functions
}
const codeStyleRules = {
  // Deprecated
  'no-alert': 'warn', // alert()
  'no-proto': 'error', // __proto__ accessor
  'no-prototype-builtins': 'error', // bans `obj.hasOwnProperty()` etc
  'no-var': 'error', // var x
  'unicorn/no-new-buffer': 'error', // NodeJS
  // '@typescript-eslint/no-namespace': 'error', // namespace T {}, they won't support type only namespace
  '@typescript-eslint/prefer-namespace-keyword': 'error', // but if you really need to, don't use `module T {}`

  // Useless code
  'no-constant-condition': 'warn', // if (false)
  'no-debugger': 'warn',
  'no-extra-bind': 'warn', // unused bind on a function that does not uses this
  'no-extra-boolean-cast': 'warn', // if (!!expr)
  'no-empty-pattern': 'warn', // const { a: {} } = expr
  'no-extra-label': 'warn', // break/continue is ok without label
  'no-unneeded-ternary': 'warn', // expr ? true : false
  'no-useless-backreference': 'error', // RegEx
  'no-useless-call': 'warn', // expr.call(undefined, ...)
  'no-useless-catch': 'warn', // catch (e) { throw e }
  'no-useless-concat': 'warn', // "a" + "b"
  'no-useless-escape': 'warn', // "hol\a"
  // 'no-lone-blocks': 'warn', // no block that not introducing a new scope
  'unicorn/no-console-spaces': 'warn', // console.log('id: ', id)
  'unicorn/no-empty-file': 'warn',
  'unicorn/no-useless-fallback-in-spread': 'warn', // {...(foo || {})}
  'unicorn/no-useless-length-check': 'warn', // array.length === 0 || array.every(...)
  'unicorn/no-useless-promise-resolve-reject': 'warn', // return Promise.resolve(value) in async function
  // 'unicorn/no-useless-spread': 'warn', // new Set([...iterable])
  'unicorn/no-zero-fractions': 'warn', // 1.0
  'unicorn/prefer-export-from': 'warn', // prefer export { } from than import-and-export
  'unicorn/prefer-native-coercion-functions': 'warn', // no coercion wrapper v => Boolean(v)
  '@typescript-eslint/await-thenable': 'warn', // await 1
  // '@typescript-eslint/no-empty-interface': 'warn', // interface T extends Q {}
  '@typescript-eslint/no-extra-non-null-assertion': 'warn', // foo!!!.bar
  // '@typescript-eslint/no-inferrable-types': 'warn', // let x: number = 1
  '@typescript-eslint/no-meaningless-void-operator': 'warn', // void a_void_call()
  '@typescript-eslint/no-non-null-asserted-nullish-coalescing': 'warn', // foo! ?? bar
  // '@typescript-eslint/no-unnecessary-boolean-literal-compare': 'warn', // no if (nullable_bool === true)
  // '@typescript-eslint/no-unnecessary-condition': 'warn', // no if (some_object)
  '@typescript-eslint/no-unnecessary-qualifier': 'warn', // no extra qualifier in enum/namespace
  '@typescript-eslint/no-unnecessary-type-arguments': 'warn', // provided type argument equals the default
  // Note: this rule seems like does not have the correct type checking behavior. before typescript-eslint has project reference support, don't use it.
  // '@typescript-eslint/no-unnecessary-type-assertion': 'warn', // non_nullable!
  '@typescript-eslint/no-unnecessary-type-constraint': 'warn', // T extends any
  // '@typescript-eslint/no-useless-constructor': 'warn', // empty constructor
  // '@typescript-eslint/no-useless-empty-export': 'warn', // export {}
  // '@typescript-eslint/no-redundant-type-constituents': 'warn', // type Q = any | T

  // Prefer modern things
  'prefer-const': 'warn',
  // 'prefer-exponentiation-operator': 'warn', // **
  // 'prefer-named-capture-group': 'warn', // RegEx
  'prefer-object-has-own': 'warn',
  // 'prefer-object-spread': 'warn', // { ... } than Object.assign
  // 'prefer-rest-params': 'warn',

  'unicorn/no-document-cookie': 'error', // even if you have to do so, use CookieJar
  'unicorn/prefer-keyboard-event-key': 'warn',
  'unicorn/prefer-add-event-listener': 'warn',
  // 'unicorn/prefer-array-find': 'warn',
  // 'unicorn/prefer-array-flat': 'warn',
  // 'unicorn/prefer-array-flat-map': 'warn',
  'unicorn/prefer-array-index-of': 'warn',
  // 'unicorn/prefer-array-some': 'warn',
  'unicorn/prefer-at': 'warn',
  'unicorn/prefer-blob-reading-methods': 'warn',
  'unicorn/prefer-date-now': 'warn',
  // 'unicorn/prefer-dom-node-append': 'warn',
  'unicorn/prefer-dom-node-dataset': 'warn',
  // 'unicorn/prefer-dom-node-remove': 'warn',
  // 'unicorn/prefer-dom-node-text-content': 'warn',
  'unicorn/prefer-event-target': 'warn', // prevent use of Node's EventEmitter
  'unicorn/prefer-math-min-max': 'warn', // Math.min/max than x < y ? x : y
  'unicorn/prefer-math-trunc': 'warn',
  'unicorn/prefer-modern-dom-apis': 'warn',
  'unicorn/prefer-modern-math-apis': 'warn',
  // 'unicorn/prefer-object-from-entries': 'warn',
  // 'unicorn/prefer-query-selector': 'warn',
  'unicorn/prefer-number-properties': 'warn',
  'unicorn/prefer-reflect-apply': 'warn',
  // 'unicorn/prefer-set-has': 'warn',
  'unicorn/prefer-set-size': 'warn',
  // 'unicorn/prefer-spread': 'warn', // prefer [...] than Array.from
  'unicorn/prefer-string-replace-all': 'warn', // str.replaceAll(...)
  'unicorn/prefer-string-slice': 'warn',
  'unicorn/prefer-string-trim-start-end': 'warn', // str.trimStart(...)
  '@typescript-eslint/no-this-alias': 'warn',
  '@typescript-eslint/prefer-string-starts-ends-with': 'warn',
  '@typescript-eslint/prefer-for-of': 'warn',
  '@typescript-eslint/prefer-includes': 'warn',
  '@typescript-eslint/no-for-in-array': 'warn',
  // '@typescript-eslint/prefer-nullish-coalescing': 'warn',
  '@typescript-eslint/prefer-optional-chain': 'warn',

  // Better debug
  // 'prefer-promise-reject-errors': 'warn', // Promise.reject(need_error)
  'symbol-description': 'warn', // Symbol(desc)
  'unicorn/catch-error-name': ['warn', { ignore: ['^err$'] }], // catch (err)
  // 'unicorn/custom-error-definition': 'warn', // correctly extends the native error
  // 'unicorn/error-message': 'warn', // error must have a message
  // 'unicorn/prefer-type-error': 'warn', // prefer TypeError
  // '@typescript-eslint/only-throw-error': 'warn', // no throw 'string'

  // API design
  // '@typescript-eslint/no-extraneous-class': 'error', // no class with only static members
  // '@typescript-eslint/prefer-readonly': 'error',
  // '@typescript-eslint/prefer-readonly-parameter-types': 'error',

  // More readable code
  // 'max-lines': ['warn', { max: 400 }],
  // 'no-dupe-else-if': 'warn', // different condition with same if body
  // 'no-else-return': 'warn',
  'no-regex-spaces': 'error', // RegEx
  'object-shorthand': 'warn',
  'prefer-numeric-literals': 'warn', // 0b111110111 === 503
  'prefer-regex-literals': 'warn', // RegEx
  'spaced-comment': ['warn', 'always', { line: { markers: ['/'] } }],
  // 'unicorn/no-array-reduce': 'warn',
  // 'unicorn/no-lonely-if': 'warn', // else if (a) { if (b) expr }
  // 'unicorn/no-negated-condition': 'warn', // if (!a) else
  // 'unicorn/no-nested-ternary': 'warn', // a ? b : c ? d : e
  // 'unicorn/no-typeof-undefined': 'warn', // typeof expr !== 'undefined'
  // 'unicorn/no-unreadable-array-destructuring': 'warn', // [,, foo] = parts
  'unicorn/no-unreadable-iife': 'warn', // (bar => (bar ? bar.baz : baz))(getBar())
  'unicorn/prefer-import-meta-properties': 'warn',
  // 'unicorn/prefer-negative-index': 'warn',
  'unicorn/prefer-single-call': 'warn',
  'unicorn/throw-new-error': 'warn',
  // 'unicorn/prefer-logical-operator-over-ternary': 'warn', // prefer ?? and ||
  // 'unicorn/prefer-optional-catch-binding': 'warn', // prefer to omit catch binding
  '@typescript-eslint/prefer-as-const': 'warn',
  // '@typescript-eslint/no-unnecessary-type-conversion': 'warn', // for code like str.toString()

  // Consistency
  'no-irregular-whitespace': 'warn', // unusual but safe
  yoda: 'warn',
  'unicorn/better-regex': 'error', // RegEx
  'unicorn/consistent-existence-index-check': 'warn', // index === -1
  'unicorn/escape-case': 'warn', // correct casing of escape '\xA9'
  'unicorn/no-hex-escape': 'warn', // correct casing of escape '\u001B'
  // 'unicorn/numeric-separators-style': 'warn', // correct using of 1_234_567
  'unicorn/prefer-prototype-methods': 'warn', // prefer Array.prototype.slice than [].slice
  'unicorn/relative-url-style': ['warn', 'always'], // prefer relative url starts with ./
  // 'unicorn/text-encoding-identifier-case': 'warn', // prefer 'utf-8' than 'UTF-8'
  '@typescript-eslint/array-type': ['warn', { default: 'array-simple' }], // prefer T[] than Array<T>
  // '@typescript-eslint/consistent-generic-constructors': 'warn', // prefer const map = new Map<string, number>() than generics on the left
  '@typescript-eslint/consistent-type-assertions': [
    'warn',
    { assertionStyle: 'as' /* objectLiteralTypeAssertions: 'never' */ },
  ], // prefer a as T than <T>a, and bans it on object literal
  // '@typescript-eslint/consistent-type-definitions': 'warn', // prefer interface, also has better performance when type checking
  '@typescript-eslint/dot-notation': 'warn', // prefer a.b than a['b']
  '@typescript-eslint/no-array-constructor': 'warn',
  // '@typescript-eslint/non-nullable-type-assertion-style': 'warn', // prefer a! than a as T
  // '@typescript-eslint/prefer-function-type': 'warn',
  '@typescript-eslint/prefer-reduce-type-parameter': 'warn',
  // '@typescript-eslint/sort-type-constituents': 'warn',
  // '@typescript-eslint/triple-slash-reference': ['error', { lib: 'never', path: 'never', types: 'always' }],
  // '@typescript-eslint/unified-signatures': 'warn', // prefer merging overload

  // Naming convention
  // 'func-name-matching': 'warn',
  // 'new-cap': 'warn',

  // Bad practice
  'no-ex-assign': 'warn', // reassign err in catch
  'no-multi-assign': 'warn', // a = b = c
  // 'no-param-reassign': 'warn',
  'no-return-assign': 'warn', // return x = expr
  'unicorn/no-object-as-default-parameter': 'warn',
  '@typescript-eslint/default-param-last': 'warn', // (a, b = 1, c)
  '@typescript-eslint/no-dynamic-delete': 'error', // this usually means you should use Map/Set
  /// Async functions / Promise bad practice
  'no-async-promise-executor': 'error', // new Promise(async (resolve) => )
  'no-promise-executor-return': 'error', // new Promise(() => result)
  // '@typescript-eslint/no-floating-promises': 'warn', // unhandled promises
  // '@typescript-eslint/promise-function-async': 'warn', // avoid Zalgo
  '@typescript-eslint/return-await': 'warn', // return await expr

  // No unused
  'no-unused-labels': 'warn',
  // 'unicorn/no-unused-properties': 'warn',
  // '@typescript-eslint/no-unused-expressions': 'warn',
  // '@typescript-eslint/no-unused-vars': 'warn',
}
const moduleSystemRules = {
  // Style
  'unused-imports/no-unused-imports': 'warn',
  'unicorn/prefer-node-protocol': 'warn',
  '@typescript-eslint/consistent-type-exports': [
    'warn',
    { fixMixedExportsWithInlineTypeSpecifier: true },
  ],
  '@typescript-eslint/consistent-type-imports': [
    'warn',
    {
      prefer: 'type-imports',
      disallowTypeAnnotations: false,
      fixStyle: 'inline-type-imports',
    },
  ],
  'no-useless-rename': 'error',
}

/** @type {any} */
const plugins = {
  unicorn: UnicornPlugin,
  '@typescript-eslint': tseslint.plugin,
  'unused-imports': UnusedImportsPlugin,
  // @ts-ignore
}
export default defineConfig(
  {
    ignores: ['**/dist/**', '**/.rollup.cache/**', '**/node_modules/**'],
  },
  {
    files: [
      'packages/**/*.ts',
      'packages/**/*.tsx',
      'apps/**/*.ts',
      'apps/**/*.tsx',
    ],
    languageOptions: {
      parser: tseslint.parser,
      parserOptions: {
        ecmaVersion: 'latest',
        projectService: true,
        // @ts-expect-error
        tsconfigRootDir: import.meta.dirname,
        warnOnUnsupportedTypeScriptVersion: false,
        allowAutomaticSingleRunInference: true,
      },
    },
    plugins,
    linterOptions: {
      reportUnusedDisableDirectives: true,
    },
    rules: /** @type {any} */ ({
      ...avoidMistakeRules,
      ...codeStyleRules,
      ...moduleSystemRules,
    }),
  },
  {
    files: [
      'packages/**/tests/**/*.ts',
      'packages/**/test/**/*.ts',
      'apps/**/tests/**/*.ts',
      'apps/**/test/**/*.ts',
    ],
    rules: {
      'unicorn/consistent-function-scoping': 'off',
    },
  },
)
