import {createI18n} from "vue-i18n";
import zhCN from './locales/zh_CN.ts';
import zhHK from './locales/zh_HK.ts';
import enUS from './locales/en_US.ts';
import jaJP from "./locales/ja_JP.ts";

export type AppLanguage = 'en_US' | 'zh_CN' | 'zh_HK' | 'ja_JP';
export const AllAppLanguages = [
    {
        title: 'English',
        value: 'en_US',
    }, {
        title: '简体中文',
        value: 'zh_CN',
    }, {
        title: '繁體中文',
        value: 'zh_HK',
    }, {
        title: '日本語',
        value: 'ja_JP',
    }
]

const i18n = createI18n({
    //  @ts-nocheck
    legacy: false,
    fallbackLocale: 'en_US',
    locale: 'en_US',
    messages: {
        'en_US': enUS,
        'zh_CN': zhCN,
        'zh_HK': zhHK,
        'ja_JP': jaJP,
    },
    warnHtmlInMessage: 'off'
})

export default i18n;