import {createVuetify} from "vuetify";
import {aliases, mdi} from "vuetify/iconsets/mdi";


export default createVuetify({
    ssr: false,
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
    defaults:{

    }
})