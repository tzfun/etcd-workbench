import {createI18n} from "vue-i18n";
import zhCN from './locales/zh_CN.ts';
import zhHK from './locales/zh_HK.ts';
import enUS from './locales/en_US.ts';

const i18n = createI18n({
    locale: 'en_US',
    messages: {
        'en_US': enUS,
        'zh_CN': zhCN,
        'zh_HK': zhHK,
    }
})

export default i18n;