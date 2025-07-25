import {createI18n} from "vue-i18n";
import zhCN from './locales/zh_CN.ts';
import enUS from './locales/en_US.ts';

const i18n = createI18n({
    locale: 'en-US',
    messages: {
        'zh-CN': zhCN,
        'en-US': enUS,
    }
})

export default i18n;