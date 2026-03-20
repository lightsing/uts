import {
  defineConfig,
  resolveSiteDataByRoute,
  type HeadConfig,
} from 'vitepress'
import {
  groupIconMdPlugin,
  groupIconVitePlugin,
} from 'vitepress-plugin-group-icons'
import llmstxt from 'vitepress-plugin-llms'

const prod = !!process.env.NETLIFY

export default defineConfig({
  title: 'UTS',
  description: 'Universal Timestamps - Decentralized timestamping protocol',

  rewrites: {
    'en/:rest*': ':rest*',
  },

  lastUpdated: true,
  cleanUrls: true,
  metaChunk: true,

  markdown: {
    math: true,
    config(md) {
      md.use(groupIconMdPlugin)
    },
  },

  sitemap: {
    hostname: 'https://docs.timestamps.now',
  },

  head: [
    ['link', { rel: 'icon', type: 'image/svg+xml', href: '/uts-logo.svg' }],
    ['meta', { name: 'theme-color', content: '#5f67ee' }],
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:site_name', content: 'UTS Documentation' }],
    ['meta', { property: 'og:url', content: 'https://docs.timestamps.now/' }],
  ],

  themeConfig: {
    logo: { src: '/uts-logo.svg', width: 24, height: 24 },

    socialLinks: [{ icon: 'github', link: 'https://github.com/lightsing/uts' }],

    search: {
      provider: 'local',
    },
  },

  locales: {
    root: { label: 'English', lang: 'en-US', dir: 'ltr' },
    zh: { label: '简体中文', lang: 'zh-Hans', dir: 'ltr' },
  },

  vite: {
    plugins: [
      groupIconVitePlugin(),
      prod &&
        llmstxt({
          workDir: 'en',
          ignoreFiles: ['index.md'],
        }),
    ],
    experimental: {
      enableNativePlugin: true,
    },
  },

  transformPageData: prod
    ? (pageData, ctx) => {
        const site = resolveSiteDataByRoute(
          ctx.siteConfig.site,
          pageData.relativePath,
        )
        const title = `${pageData.title || site.title} | ${pageData.description || site.description}`
        ;((pageData.frontmatter.head ??= []) as HeadConfig[]).push(
          ['meta', { property: 'og:locale', content: site.lang }],
          ['meta', { property: 'og:title', content: title }],
        )
      }
    : undefined,
})
