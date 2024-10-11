import { defineConfig } from 'vitepress'
import {enConfig} from "./config/en.js";
import {zhConfig} from "./config/zh.js";

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "Etcd Workbench",
  description: "A beautiful and lightweight ETCD client",
  base:"/etcd-workbench/",
  themeConfig: {
    logo: '/images/logo.png',
    siteTitle: 'Etcd Workbench',

    sidebar: [],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/tzfun/etcd-workbench' },
    ],

    search: {
      provider: 'local'
    }
  },
  locales: {
    root: {
      label: 'English',
      lang: 'en',
      ...enConfig
    },
    zh: {
      label: '中文',
      lang: 'zh',
      ...zhConfig
    },
  },
})
