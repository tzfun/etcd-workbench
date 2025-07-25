import {createApp} from "vue";
import App from "./App.vue";
import vuetify from "~/common/vuetify.ts";
import '@mdi/font/css/materialdesignicons.css'
import '~/styles/main.scss'
import {_onClientError} from "~/common/windows.ts";
import {_goBrowserPage} from "~/common/utils.ts";
import {_useLocalEvents} from "~/common/events.ts";
import i18n from "~/language";

const setup = () => {
    try {
        //  @ts-ignore
        window._goBrowserPage = _goBrowserPage
        //  @ts-ignore
        window._localEvent = _useLocalEvents()
        const app = createApp(App)
        app.use(vuetify)
        app.use(i18n)

        app.config.errorHandler = (err: any, instance, info) => {
            console.error(err, info, instance)
            let exit: boolean = !!(instance
                && instance.$.type.__name?.match("AppMain|App")
                && info.match("mounted|setup|create"))

            _onClientError(info, err.stack, exit)
        }

        app.mount("#app");
    } catch (e: any) {
        console.error(e)
        _onClientError("Setup failed", e.stack, true)
    }
}
setup()