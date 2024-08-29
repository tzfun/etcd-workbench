<script setup lang="ts">
import {appWindow} from '@tauri-apps/api/window'
import {AppTheme, DialogItem, TipsItem} from "~/common/types.ts";
import {computed, onMounted, onUnmounted, reactive, ref, watch} from "vue";
import {platform as getPlatform} from "@tauri-apps/api/os";
import {useTheme} from "vuetify";
import WindowsSystemBar from "~/components/system-bar/WindowsSystemBar.vue";
import MacSystemBar from "~/components/system-bar/MacSystemBar.vue";
import AppSetting from "~/pages/setting/AppSetting.vue";
import AppMain from "~/pages/main/AppMain.vue";
import {_checkUpdate, _listenLocal, EventName} from "~/common/events.ts";
import {_loadSettings, _useSettings, _useUpdateInfo} from "~/common/store.ts";
import {DEFAULT_SETTING_CONFIG} from "~/common/transport/setting.ts";
import {installUpdate} from "@tauri-apps/api/updater";

const loading = ref(false)
const dialogs = ref<DialogItem[]>([])
const tips = ref<TipsItem[]>([])
const platform = ref<string>('win32')

const theme = useTheme()

const eventUnListens = reactive<Function[]>([])

const windowLabel = computed<string>(() => {
  return appWindow.label
})

onMounted(async () => {
  let settings = await _loadSettings()

  //  频闭Webview原生事件
  disableWebviewNativeEvents()

  setAppTheme(settings.theme)

  eventUnListens.push(await appWindow.listen('tauri://theme-changed', (e) => {
    let systemTheme = e.payload as string
    if (systemTheme) {
      theme.global.name.value = systemTheme
    }
  }))

  eventUnListens.push(await appWindow.listen('tauri://menu', (e) => {
    console.log('menu', e.payload)
  }))

  platform.value = await getPlatform()
  if (platform.value != 'win32') {
    document.getElementById("app")!.classList.add("main-window-radius")
  }

  _listenLocal(EventName.LOADING, (e) => {
    loading.value = e as boolean
  })

  _listenLocal(EventName.DIALOG, (e) => {
    let dialog = e as DialogItem
    let idx = -1;
    for (let i = 0; i < dialogs.value.length; i++) {
      if (!dialogs.value[i].value) {
        idx = i;
        break
      }
    }

    dialog.value = true
    if (idx < 0) {
      dialogs.value.push(dialog)
    } else {
      dialogs.value[idx] = dialog
    }
  })

  _listenLocal(EventName.TIP, (e) => {
    let tip = e as TipsItem
    let idx = -1;
    for (let i = 0; i < tips.value.length; i++) {
      if (!tips.value[i].value) {
        idx = i;
        break
      }
    }

    tip.value = true
    if (idx < 0) {
      tips.value.push(tip)
    } else {
      tips.value[idx] = tip
    }
  })

  if (windowLabel.value == 'main') {
    watch(() => _useSettings().value, (newVal, oldVal) => {
      if (oldVal == DEFAULT_SETTING_CONFIG) {
        return
      }
      if (newVal.theme !== oldVal.theme) {
        setAppTheme(newVal.theme)
      }
    })

    let updateInfo = _useUpdateInfo().value
    // 检查更新
    _checkUpdate().then(async (manifest) => {
      console.log(manifest,settings.autoUpdate)
      updateInfo.valid = true
      updateInfo.latestVersion = manifest

      if (settings.autoUpdate) {
        await installUpdate()
      }
    }).catch(() => {
      updateInfo.valid = false
    })
  }
})

onUnmounted(() => {
  for (let eventUnListen of eventUnListens) {
    eventUnListen()
  }
})

const setAppTheme = (appTheme: AppTheme) => {
  if (appTheme == 'auto') {
    appWindow.theme().then(systemTheme => {
      if (systemTheme) {
        theme.global.name.value = systemTheme
      }
    })
  } else {
    theme.global.name.value = appTheme
  }
}

const disableWebviewNativeEvents = () => {
  document.addEventListener('keydown', e => {
    let key = e.key.toLowerCase()
    if (e.ctrlKey && key == 'r') {
      e.preventDefault()
      return false
    }

    if (key == 'f5' || key == 'escape') {
      e.preventDefault()
      return false
    }
  }, {capture: true})

  document.addEventListener('contextmenu', e => {
    e.preventDefault()
    return false
  }, {capture: true})
}

</script>

<template>
  <v-app id="vuetify-app">
    <v-layout>
      <WindowsSystemBar v-if="platform == 'win32'"
                        :height="28"
                        :window-label="windowLabel"
      ></WindowsSystemBar>
      <MacSystemBar v-if="platform == 'darwin'"
                    :height="28"
                    :window-label="windowLabel"
      ></MacSystemBar>

      <v-main class="fill-height position-relative" id="mainBody">
        <AppSetting v-if="windowLabel === 'setting'"
                    class="app-setting"
                    :platform="platform"
        ></AppSetting>
        <AppMain v-else-if="windowLabel === 'main'"
                 :platform="platform"
        ></AppMain>

      </v-main>
    </v-layout>

    <!--    全局公共组件    -->

    <v-overlay
        :model-value="loading"
        persistent
        data-tauri-drag-region
        attach="#mainBody"
        class="align-center justify-center"
    >
      <v-progress-circular
          color="primary"
          size="64"
          indeterminate
      ></v-progress-circular>
    </v-overlay>

    <v-dialog
        v-for="(item, key) in dialogs"
        :key="key"
        v-model="item.value"
        :persistent="item.persistent == undefined ? true : item.persistent"
        :scrollable="item.scrollable == undefined ? true : item.scrollable"
        width="auto"
    >
      <v-card
          :max-width="item.maxWidth ? item.maxWidth : 500"
          :min-width="item.minWidth ? item.minWidth : 400"
          :title="item.title"
      >
        <template v-slot:text>
          <div v-html="item.content"></div>
        </template>

        <template v-slot:prepend>
          <v-icon :color="item.iconColor">{{ item.icon }}</v-icon>
        </template>
        <template v-slot:append v-if="item.closeBtn">
          <v-icon class="cursor-pointer" @click="item.value = false">mdi-close</v-icon>
        </template>
        <template v-slot:actions v-if="item.buttons">
          <v-btn
              v-for="(btn,k ) in item.buttons"
              :key="k"
              :class="(btn.class ? btn.class : '') + ' text-none'"
              :text="btn.text"
              :variant="btn.variant ? btn.variant : 'text'"
              :color="btn.color"
              @click="btn.callback(item, $event)"
          ></v-btn>
        </template>
      </v-card>
    </v-dialog>

    <v-snackbar
        v-for="(item, key) in tips"
        :key="key"
        v-model="item.value"
        location="top"
        class="mt-12"
        :content-class="item.class"
    >
      <v-icon v-if="item.icon" class="mr-2">{{ item.icon }}</v-icon>
      {{ item.content }}

      <template v-slot:actions v-if="item.close">
        <v-btn
            color="pink"
            variant="text"
            icon="mdi-close"
            @click="item.value = false; item.close()"
        />
      </template>
    </v-snackbar>
  </v-app>
</template>

<style scoped lang="scss">
.app-setting {
  width: 100%;
  height: calc(100% - 28px);
  position: absolute;
  z-index: 10000;
  top: 28px;
  left: 0;
}
</style>

<style lang="scss">
.v-overlay, .v-overlay__scrim {
  margin-top: 28px;
}
</style>
