import { createApp } from "vue";
import App from "./App.vue";
import vuetify from "~/common/vuetify.ts";

import '@mdi/font/css/materialdesignicons.css'
import '~/styles/main.scss'

createApp(App)
    .use(vuetify)
    .mount("#app");
