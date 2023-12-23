import 'hacktimer'
import { createApp } from "vue";
import App from "./App.vue";
import 'default-passive-events'

// import "~/styles/element/index.scss";

import "~/styles/index.scss";
import "uno.css";

import "element-plus/theme-chalk/src/message.scss";

const app = createApp(App);
app.mount("#app");
