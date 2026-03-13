export default defineAppConfig({
  ui: {
    colors: {
      primary: 'violet',
      neutral: 'slate',
    },
    footer: {
      slots: {
        root: 'border-t border-default',
        left: 'text-sm text-muted',
      },
    },
  },
  seo: {
    siteName: 'UTS Documentation',
  },
  header: {
    title: 'UTS',
    to: '/',
    logo: {
      alt: 'UTS',
      light: '',
      dark: '',
    },
    search: true,
    colorMode: true,
    links: [
      {
        icon: 'i-simple-icons-github',
        to: 'https://github.com/lightsing/uts',
        target: '_blank',
        'aria-label': 'GitHub',
      },
    ],
  },
  footer: {
    credits: `Universal Timestamps • © ${new Date().getFullYear()}`,
    colorMode: false,
    links: [
      {
        icon: 'i-simple-icons-discord',
        to: 'https://discord.gg/nuxt',
        target: '_blank',
        'aria-label': 'Discord',
      },
      {
        icon: 'i-simple-icons-x',
        to: 'https://x.com/nuxt_js',
        target: '_blank',
        'aria-label': 'X',
      },
      {
        icon: 'i-simple-icons-github',
        to: 'https://github.com/lightsing/uts',
        target: '_blank',
        'aria-label': 'GitHub',
      },
    ],
  },
  toc: {
    title: 'On this page',
    bottom: {
      title: 'Links',
      edit: 'https://github.com/lightsing/uts/edit/main/apps/docs/content',
      links: [
        {
          icon: 'i-lucide-star',
          label: 'Star on GitHub',
          to: 'https://github.com/lightsing/uts',
          target: '_blank',
        },
        {
          icon: 'i-lucide-book-open',
          label: 'EAS Documentation',
          to: 'https://attest.sh/docs/',
          target: '_blank',
        },
      ],
    },
  },
})
