import { createRequire } from 'module'
import { defineAdditionalConfig, type DefaultTheme } from 'vitepress'

const require = createRequire(import.meta.url)

export default defineAdditionalConfig({
  description: 'Universal Timestamps - Decentralized timestamping protocol',

  themeConfig: {
    nav: nav(),

    sidebar: {
      '/guide/': { base: '/guide/', items: sidebarGuide() },
      '/developer/': { base: '/developer/', items: sidebarDeveloper() },
      '/reference/': { base: '/reference/', items: sidebarReference() },
    },

    editLink: {
      pattern: 'https://github.com/lightsing/uts/edit/main/apps/docs/:path',
      text: 'Edit this page on GitHub',
    },

    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright © 2026-present Universal Timestamps',
    },
  },
})

function nav(): DefaultTheme.NavItem[] {
  return [
    {
      text: 'Guide',
      link: '/guide/what-is-uts',
      activeMatch: '/guide/',
    },
    {
      text: 'Developer',
      link: '/developer/overview',
      activeMatch: '/developer/',
    },
    {
      text: 'Reference',
      link: '/reference/further-reading',
      activeMatch: '/reference/',
    },
  ]
}

function sidebarGuide(): DefaultTheme.SidebarItem[] {
  return [
    {
      text: 'Getting Started',
      collapsed: false,
      items: [
        { text: 'What is UTS?', link: 'what-is-uts' },
        { text: 'Stamp via CLI', link: 'stamp-via-cli' },
      ],
    },
  ]
}

function sidebarDeveloper(): DefaultTheme.SidebarItem[] {
  return [
    {
      text: 'Developer Guide',
      collapsed: false,
      items: [{ text: 'Architecture Overview', link: 'overview' }],
    },
  ]
}

function sidebarReference(): DefaultTheme.SidebarItem[] {
  return [
    {
      text: 'API Reference',
      collapsed: false,
      items: [{ text: 'Further Reading', link: 'further-reading' }],
    },
  ]
}
