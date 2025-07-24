<script setup lang="ts">
import {platform as getPlatform} from "@tauri-apps/api/os";
import {appWindow} from '@tauri-apps/api/window';
import {computed, onMounted, onUnmounted, reactive, ref, watch} from "vue";
import {useTheme} from "vuetify";
import {
  _alertError,
  _confirm,
  _confirmUpdateApp,
  _genNewVersionUpdateMessage,
  _listenLocal,
  _unListenLocal,
  EventName,
  UpdateDownloadingProgressEvent
} from "~/common/events.ts";
import {_isDebugModel} from "~/common/services.ts";
import {_loadAppVersion, _loadGlobalStore, _loadSettings, _useSettings} from "~/common/store.ts";
import {DEFAULT_SETTING_CONFIG} from "~/common/transport/setting.ts";
import {AppTheme, DialogItem, TipsItem, UpdateInfo} from "~/common/types.ts";
import {_isLinux, _isMac, _isWindows, _setPlatform} from "~/common/windows.ts";
import LinuxSystemBar from "~/components/system-bar/LinuxSystemBar.vue";
import MacSystemBar from "~/components/system-bar/MacSystemBar.vue";
import WindowsSystemBar from "~/components/system-bar/WindowsSystemBar.vue";
import AppMain from "~/pages/main/AppMain.vue";
import AppSetting from "~/pages/setting/AppSetting.vue";
import {_installUpdate, CustomUpdateManifest} from './common/updater';
import IconEtcd from "~/components/icon/IconEtcd.vue";
import {Handler} from "mitt";
import {_byteTextFormat} from "~/common/utils.ts";
import {relaunch} from "@tauri-apps/api/process";

const DEFAULT_LOADING_TEXT: string = "Loading..."
const loading = ref<boolean>(false)
const loadingText = ref<string>(DEFAULT_LOADING_TEXT)

const dialogs = ref<DialogItem[]>([])
const tipsCounter = ref<number>(0)
const tips = ref<TipsItem[]>([])

const theme = useTheme()

const eventUnListens = reactive<Function[]>([])
const updateInfo = reactive<UpdateInfo>({
  state: 'none',
  chunkLength: 0,
  contentLength: 0,
  error: ''
})

const windowLabel = computed<string>(() => {
  return appWindow.label
})
const updaterDialogShow = computed<boolean>(() => {
  return updateInfo.state == 'pending'
      || updateInfo.state == 'downloading'
      || updateInfo.state == 'downloaded'
      || updateInfo.state == 'error'
})
const downloadingProgress = computed(() => {
  if (updateInfo.state == 'downloading') {
    if (updateInfo.contentLength && updateInfo.contentLength > 0) {
      return 100 * updateInfo.chunkLength / updateInfo.contentLength
    }
  }
  return 0
})

onMounted(async () => {
  await _loadAppVersion()
  await _loadGlobalStore()
  let settings = await _loadSettings()
  _setPlatform(await getPlatform())
  await listenUpdaterEvent()

  let isDebug = await _isDebugModel()
  if (!isDebug) {
    //  频闭Webview原生事件
    disableWebviewNativeEvents()
  }

  document.addEventListener('contextmenu', (e: MouseEvent) => {
    e.preventDefault()
    return false
  }, {capture: true})

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
  }
})

onUnmounted(() => {
  for (let eventUnListen of eventUnListens) {
    eventUnListen()
  }
})

const listenUpdaterEvent = async () => {
  eventUnListens.push(await appWindow.listen<CustomUpdateManifest>(EventName.UPDATE_AVAILABLE, e => {
    if (updateInfo.state != 'pending') {
      updateInfo.state = 'available'
    }

    updateInfo.chunkLength = 0;
    updateInfo.contentLength = 0;
    updateInfo.error = '';

    const autoUpdate = _useSettings().value.autoUpdate
    if (!autoUpdate && e.payload.source == appWindow.label) {
      let manifest = e.payload

      let message = _genNewVersionUpdateMessage(manifest)
      _confirmUpdateApp(message).then(() => {
        _installUpdate().then(() => {
        }).catch(e => {
          console.error(e)
          _alertError("Unable to update: " + e)
        })
      }).catch(() => {
      })
    }
  }))

  if (windowLabel.value == 'main') {
    eventUnListens.push(await appWindow.listen(EventName.UPDATE_PENDING, () => {
      updateInfo.state = 'pending'
    }))

    eventUnListens.push(await appWindow.listen<UpdateDownloadingProgressEvent>(EventName.UPDATE_DOWNLOADING_PROGRESS, e => {
      updateInfo.state = 'downloading'
      updateInfo.chunkLength += e.payload.chunkLength
      if (e.payload.contentLength) {
        updateInfo.contentLength = e.payload.contentLength
      }
    }))

    eventUnListens.push(await appWindow.listen(EventName.UPDATE_DOWNLOADED, () => {
      updateInfo.state = 'downloaded'
    }))

    eventUnListens.push(await appWindow.listen(EventName.UPDATE_INSTALLED, () => {
      updateInfo.state = 'installed'
      _confirm('Restart', 'Update complete. Restart now to apply?').then(() => {
        relaunch().catch(e => {
          console.error("Restart failed", e)
        })
      }).catch(() => {
      })
    }))

    eventUnListens.push(await appWindow.listen<string>(EventName.UPDATE_ERRORS, e => {
      console.error('update error', e.payload)
      updateInfo.state = 'error'
      updateInfo.error = e.payload

      _alertError(`An exception occurred during the update: ${e.payload}`)
    }))
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
      console.log("pass a", key)
      e.preventDefault()
      return false
    }

    if (key.match(/^f\d+$/)) {
      console.log("pass c")
      e.preventDefault()
      return false
    }
  }, {capture: true})
}

</script>

<template>
  <v-app id="vuetify-app">
    <v-layout>
      <WindowsSystemBar v-if="_isWindows()"
                        :height="28"
                        :window-label="windowLabel"
                        :update-info="updateInfo"
      />
      <MacSystemBar v-if="_isMac()"
                    :height="28"
                    :window-label="windowLabel"
                    :update-info="updateInfo"
      />
      <LinuxSystemBar v-if="_isLinux()"
                      :window-label="windowLabel"
                      :height="28"
                      :update-info="updateInfo"
      />

      <v-main class="fill-height position-relative" id="mainBody">
        <AppSetting v-if="windowLabel === 'setting'" class="app-setting"></AppSetting>
        <AppMain v-else-if="windowLabel === 'main'"></AppMain>
      </v-main>
    </v-layout>

    <!--    更新组件       -->
    <div class="updater" v-if="updaterDialogShow" data-tauri-drag-region>
      <v-list
          class="py-2"
          color="primary"
          elevation="12"
          rounded="lg"
          width="450px"
      >
        <v-list-item>
          <template v-slot:prepend>
            <v-icon v-if="updateInfo.state == 'pending'"
            >mdi-arrow-down-bold
            </v-icon>
            <v-icon v-else-if="updateInfo.state == 'downloading'"
                    class="downloading-icon"
            >mdi-arrow-down-bold
            </v-icon>
            <v-icon v-else-if="updateInfo.state == 'downloaded'"
                    color="green"
            >mdi-check-circle-outline
            </v-icon>
            <v-icon v-else-if="updateInfo.state == 'error'"
                    color="red"
            >mdi-alert-circle
            </v-icon>
          </template>
          <template v-slot:default>
            <span v-if="updateInfo.state == 'pending'">Preparing the update...</span>
            <div v-else-if="updateInfo.state == 'downloading'">
              <v-progress-linear
                  v-model="downloadingProgress"
                  color="blue-lighten-1"
                  class="my-2"
                  height="20"
              >
                <strong>{{ Math.ceil(downloadingProgress) }}%</strong>
              </v-progress-linear>
              <v-layout>
                Downloading...
                <v-spacer/>
                <span class="text-medium-emphasis">
                  {{ _byteTextFormat(updateInfo.chunkLength) }} / {{ _byteTextFormat(updateInfo.contentLength) }}
                </span>

              </v-layout>
            </div>
            <span v-else-if="updateInfo.state == 'downloaded'">
              Download completed. Installing now...
            </span>
            <v-layout v-else-if="updateInfo.state == 'error'">
              Failed to update. {{ updateInfo.error }}
              <v-spacer/>
              <v-icon @click="updateInfo.state = 'none'">mdi-close</v-icon>
            </v-layout>
          </template>
        </v-list-item>
      </v-list>
    </div>

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
        :style="`z-index: ${item.zIndex ? item.zIndex : 2000};`"
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
        style="z-index: 3000;"
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

.updater {
  position: fixed;
  left: 0;
  top: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(255, 255, 255, 0.32);
  z-index: 1500;
  display: flex;
  justify-content: center;
  align-content: center;
  flex-wrap: wrap;

  .downloading-icon {
    animation: downloading 1s linear infinite;
  }
}

@keyframes downloading {
  0% {
    transform: translateY(-50%);
    opacity: 0;
  }
  50% {
    opacity: 1;
  }
  100% {
    transform: translateY(50%);
    opacity: 0;
  }
}
</style>

<style lang="scss">

</style>
