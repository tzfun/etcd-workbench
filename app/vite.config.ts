import {defineConfig} from "vite";
import vue from "@vitejs/plugin-vue";
import vuetify from 'vite-plugin-vuetify';
import {join} from "path";
import {visualizer} from 'rollup-plugin-visualizer';
// import inject from '@rollup/plugin-inject'

// https://vitejs.dev/config/
export default defineConfig(async () => ({
    plugins: [
        vue(),
        vuetify({
            styles: {
                configFile: 'src/styles/settings.scss'
            }
        }),
        visualizer({ open: true }), // 自动开启分析页面
        // inject({
            // $: 'jquery',
            // jQuery: 'jquery',
            // "windows.jQuery": "jquery"
        // })
    ],
    // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
    //
    // 1. prevent vite from obscuring rust errors
    clearScreen: false,
    // 2. tauri expects a fixed port, fail if that port is not available
    server: {
        port: 1420,
        strictPort: true,
        watch: {
            // 3. tell vite to ignore watching `src-tauri`
            ignored: ["**/src-tauri/**"],
        },
    },
    resolve: {
        alias: {
            '~': join(__dirname, "src")
        },
    },
    build: {
        rollupOptions: {
            input: {
                index: "./index.html",
                splashscreen: "./splashscreen.html",
            },
        }
    },
    optimizeDeps: {
        include: ['jquery']
    }
}));
