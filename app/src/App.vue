<script setup lang="ts">
import { platform as getPlatform } from "@tauri-apps/api/os";
import { relaunch } from "@tauri-apps/api/process";
import { appWindow } from '@tauri-apps/api/window';
import { computed, onMounted, onUnmounted, reactive, ref, watch } from "vue";
import { useTheme } from "vuetify";
import {
  _alertError,
  _listenLocal,
  _loading, _unListenLocal,
  EventName
} from "~/common/events.ts";
import { _isDebugModel } from "~/common/services.ts";
import { _loadAppVersion, _loadGlobalStore, _loadSettings, _useSettings, _useUpdateInfo } from "~/common/store.ts";
import { DEFAULT_SETTING_CONFIG } from "~/common/transport/setting.ts";
import { AppTheme, DialogItem, TipsItem } from "~/common/types.ts";
import { _isLinux, _isMac, _isWindows, _setPlatform } from "~/common/windows.ts";
import LinuxSystemBar from "~/components/system-bar/LinuxSystemBar.vue";
import MacSystemBar from "~/components/system-bar/MacSystemBar.vue";
import WindowsSystemBar from "~/components/system-bar/WindowsSystemBar.vue";
import AppMain from "~/pages/main/AppMain.vue";
import AppSetting from "~/pages/setting/AppSetting.vue";
import { _checkUpdate, _installUpdate } from './common/updater';
import IconEtcd from "~/components/icon/IconEtcd.vue";
import {Handler} from "mitt";

const DEFAULT_LOADING_TEXT: string = "Loading..."
const loading = ref<boolean>(false)
const loadingText = ref<string>(DEFAULT_LOADING_TEXT)

const dialogs = ref<DialogItem[]>([])
const tipsCounter = ref<number>(0)
const tips = ref<TipsItem[]>([])

const theme = useTheme()

const eventUnListens = reactive<Function[]>([])
const checkUpdateTimer = ref<number>()

const windowLabel = computed<string>(() => {
  return appWindow.label
})

onMounted(async () => {
  await _loadAppVersion()
  await _loadGlobalStore()
  let settings = await _loadSettings()
  _setPlatform(await getPlatform())

  let isDebug = await _isDebugModel()
  if (!isDebug) {
    //  频闭Webview原生事件
    disableWebviewNativeEvents()
  }

  setAppTheme(settings.theme)

  eventUnListens.push(await appWindow.listen('tauri://theme-changed', (e) => {
    let systemTheme = e.payload as string
    if (systemTheme) {
      theme.global.name.value = systemTheme
    }
  }))

  //  mac和linux添加窗口圆角
  if (_isMac() || _isLinux()) {
    document.getElementById("app")!.classList.add("main-window-radius")
  }
  const loadingEventHandler: Handler<any> = (e: { state: boolean, text: string | undefined }) => {
    loadingText.value = e.text || DEFAULT_LOADING_TEXT
    loading.value = e.state
  }
  _listenLocal(EventName.LOADING, loadingEventHandler)
  eventUnListens.push(() => {
    _unListenLocal(EventName.LOADING, loadingEventHandler)
  })

  const dialogEventHandler: Handler<any> = (e) => {
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
  }
  _listenLocal(EventName.DIALOG, dialogEventHandler)
  eventUnListens.push(() => {
    _unListenLocal(EventName.DIALOG, dialogEventHandler)
  })

  const tipEventHandler: Handler<any> = (e) => {
    let tip = e as TipsItem
    let idx = -1;
    for (let i = 0; i < tips.value.length; i++) {
      if (!tips.value[i].value) {
        idx = i;
        break
      }
    }

    tip.id = ++tipsCounter.value
    tip.value = true
    if (idx < 0) {
      tips.value.push(tip)
    } else {
      tips.value[idx] = tip
    }
  }
  _listenLocal(EventName.TIP, tipEventHandler)
  eventUnListens.push(() => {
    _unListenLocal(EventName.TIP, tipEventHandler)
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

    checkUpdate(settings.autoUpdate)

    startCheckUpdateTimer()
  }
})

onUnmounted(() => {
  for (let eventUnListen of eventUnListens) {
    eventUnListen()
  }
  stopCheckUpdateTimer()
})

const checkUpdate = (autoUpdate: boolean) => {
  let updateInfo = _useUpdateInfo().value
  // 检查更新
  _checkUpdate().then(async (manifest) => {
    updateInfo.valid = true
    updateInfo.latestVersion = manifest

    if (autoUpdate) {
      _loading(true, "Installing new version...")
      _installUpdate().then(() => {
        _loading(true, "Relaunch...")
        relaunch().catch((e: string) => {
          console.error(e)
          _alertError("Unable to relaunch, please relaunch manually.")
        }).finally(() => {
          _loading(false)
        })
      }).catch(e => {
        _loading(false)
        console.error(e)
        _alertError(`Install failed, please update manually or go to <span onclick='_goBrowserPage("https://github.com/tzfun/etcd-workbench")' class='simulate-tag-a text-green font-weight-bold' title='Click to view github'>GitHub</span> to download the latest version.`)
      })
    }
  }).catch(e => {
    console.error("Failed to check update", e)
    updateInfo.valid = false
  })
}

const startCheckUpdateTimer = () => {
  stopCheckUpdateTimer()
  //  每1小时检查一次更新
  checkUpdateTimer.value = window.setInterval(() => {
    checkUpdate(false)
  }, 3600_000)
}

const stopCheckUpdateTimer = () => {
  if (checkUpdateTimer.value) {
    clearInterval(checkUpdateTimer.value)
  }
}

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

    if (e.ctrlKey && e.shiftKey && key == 'i') {
      console.log("pass b")
      e.preventDefault()
      return false
    }
    //  阻止快捷键：
    //  ctrl + p
    //  ctrl + r
    //  ctrl + i
    //  ctrl + u
    //  ctrl + j
    if (e.ctrlKey && /^[priuj]$/.test(key)) {
      console.log("pass a",key)
      e.preventDefault()
      return false
    }

    if (key.match(/^f\d+$/)) {
      console.log("pass c")
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
      <WindowsSystemBar v-if="_isWindows()"
                        :height="28"
                        :window-label="windowLabel"
      ></WindowsSystemBar>
      <MacSystemBar v-if="_isMac()"
                    :height="28"
                    :window-label="windowLabel"
      ></MacSystemBar>
      <LinuxSystemBar v-if="_isLinux()"
                      :window-label="windowLabel"
                      :height="28"
      ></LinuxSystemBar>

      <v-main class="fill-height position-relative" id="mainBody">
        <AppSetting v-if="windowLabel === 'setting'" class="app-setting"></AppSetting>
        <AppMain v-else-if="windowLabel === 'main'"></AppMain>
      </v-main>
    </v-layout>

    <!--    全局公共组件    -->

    <v-dialog
        v-model="loading"
        data-tauri-drag-region
        max-width="320"
        persistent
        style="z-index: 2000;"
    >
      <v-list
          class="py-2"
          color="primary"
          elevation="12"
          rounded="lg"
      >
        <v-list-item
            :title="loadingText"
        >
          <template v-slot:prepend>
            <div class="pe-4">
              <IconEtcd/>
            </div>
          </template>

          <template v-slot:append>
            <v-progress-circular
                color="primary"
                indeterminate="disable-shrink"
                size="16"
                width="2"
            ></v-progress-circular>
          </template>
        </v-list-item>
      </v-list>
    </v-dialog>

    <v-dialog
        v-for="(item, key) in dialogs"
        :key="key"
        v-model="item.value"
        :persistent="item.persistent == undefined ? true : item.persistent"
        :scrollable="item.scrollable == undefined ? true : item.scrollable"
        width="auto"
        style="z-index: 2000;"
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
          <v-icon :color="item.iconColor" v-if="item.icon">{{ item.icon }}</v-icon>
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
        v-for="item in tips"
        :key="item.id"
        v-model="item.value"
        location="top"
        class="mt-12"
        :content-class="item.class"
        :timeout="item.timeout"
        style="z-index: 2000;"
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
  z-index: 1;
  top: 28px;
  left: 0;
}
</style>

<style lang="scss">

</style>
