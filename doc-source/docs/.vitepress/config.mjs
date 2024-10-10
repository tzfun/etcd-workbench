import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "Etcd Workbench",
  description: "A beautiful and lightweight ETCD client",
  themeConfig: {
    logo: '/images/logo.png',
    siteTitle: 'Etcd Workbench',

    // https://vitepress.dev/reference/default-theme-config
    nav: [
      { text: 'Home', link: '/' },
      { text: 'Examples', link: '/markdown-examples' }
    ],

    sidebar: [
      {
        text: 'Examples',
        items: [
          { text: 'Markdown Examples', link: '/markdown-examples' },
          { text: 'Runtime API Examples', link: '/api-examples' }
        ]
      }
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/tzfun/etcd-workbench' }
    ],

    search: {
      provider: 'local'
    }
  }
})
