import { createApp } from "vue";
import App from "./App.vue";

// import "~/styles/element/index.scss";

import "~/styles/index.scss";
import "uno.css";

// If you want to use ElMessage, import it.
import "element-plus/theme-chalk/src/message.scss";

const app = createApp(App);
app.mount("#app");
