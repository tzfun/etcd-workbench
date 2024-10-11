export const META_URL = 'https://etcd.beifengtz.com'
export const META_TITLE = 'Etcd Workbench'
export const META_DESCRIPTION = '一个漂亮、轻量的 ETCD V3 客户端，提供 App 和 Web 版本，支持SSL、SSH Tunnel连接。'

export const zhConfig = {
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
            // {text: 'Home', link: '/zh/'},
        ],
        footer: {
            message: '基于Apache-2.0开源许可协议',
            copyright: 'Copyright © 2023-present Tiny Craft'
        }
    }
}