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

export type ButtonSize = 'x-small' | 'small' | 'large' | 'x-large' | undefined
export type Density = 'default' | 'comfortable' | 'compact'

export const CONNECTION_LIST_DENSITY: Density = 'compact'

//  每个页面刷新按钮大小
export const PAGE_REFRESH_BUTTON_SIZE: ButtonSize = 'small'
//  每个页面主按钮大小
export const PAGE_BUTTON_SIZE: ButtonSize = undefined

//  KV编辑器按钮大小
export const KEY_EDITOR_BUTTON_SIZE: ButtonSize = 'small'
//  KV编辑器按钮边距
export const KEY_EDITOR_BUTTON_DENSITY: Density = 'comfortable'

//  对话框按钮大小
export const DIALOG_BUTTON_SIZE: ButtonSize = undefined
//  对话框按钮边距
export const DIALOG_BUTTON_DENSITY: Density = 'comfortable'