import {createVuetify} from "vuetify";
import {aliases, mdi} from "vuetify/iconsets/mdi";

import {VTreeview} from "vuetify/labs/components";

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
    defaults: {}
})