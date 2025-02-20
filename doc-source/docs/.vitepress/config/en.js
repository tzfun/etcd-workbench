export const META_URL = 'https://etcd.beifengtz.com'
export const META_TITLE = 'Etcd Workbench'
export const META_DESCRIPTION = 'A beautiful and lightweight ETCD V3 client. Provides App and Web packages. Supports SSL and SSH Tunnel connections.'

export const enConfig = {
    description: META_DESCRIPTION,
    head: [
        ['meta', {property: 'og:url', content: META_URL}],
        ['meta', {property: 'og:description', content: META_DESCRIPTION}],
        ['meta', {property: 'twitter:url', content: META_URL}],
        ['meta', {property: 'twitter:title', content: META_TITLE}],
        ['meta', {property: 'twitter:description', content: META_DESCRIPTION}],
    ],
    themeConfig: {
        nav: [
            // {text: 'Home', link: '/'},
        ],
        footer: {
            message: 'Released under the GPL-3.0 License.',
            copyright: 'Copyright © 2023-present beifengtz'
        }
    }
}