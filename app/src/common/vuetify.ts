import {createVuetify} from "vuetify";
import {aliases, mdi} from "vuetify/iconsets/mdi";

import {VTreeview} from "vuetify/labs/components";
import {createVueI18nAdapter} from "vuetify/locale/adapters/vue-i18n";
import i18n from "~/language";
import {useI18n} from "vue-i18n";

export default createVuetify({
    ssr: false,
    components: {
        VTreeview
    },
    icons: {
        defaultSet: 'mdi',
        aliases,
        sets: {
            mdi,
        },
    },
    theme: {
        defaultTheme: 'light',
        themes: {},
    },
    locale: {
        adapter: createVueI18nAdapter({
            //  @ts-ignore
            i18n,
            useI18n
        })
    },
    defaults: {}
})