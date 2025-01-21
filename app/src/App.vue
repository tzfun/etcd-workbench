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
import {
  _alertError,
  _checkUpdate,
  _confirmUpdateApp,
  _genNewVersionUpdateMessage,
  _listenLocal,
  _loading,
  EventName
} from "~/common/events.ts";
import {_loadAppVersion, _loadGlobalStore, _loadSettings, _useSettings, _useUpdateInfo} from "~/common/store.ts";
import {DEFAULT_SETTING_CONFIG} from "~/common/transport/setting.ts";
import {installUpdate} from "@tauri-apps/api/updater";
import {_isDebugModel} from "~/common/services.ts";
import LinuxSystemBar from "~/components/system-bar/LinuxSystemBar.vue";
import {_isLinux, _isMac, _isWindows, _setPlatform} from "~/common/windows.ts";
import {relaunch} from "@tauri-apps/api/process";

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

  _listenLocal(EventName.LOADING, (e: { state: boolean, text: string | undefined }) => {
    loadingText.value = e.text || DEFAULT_LOADING_TEXT
    loading.value = e.state
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

    tip.id = ++tipsCounter.value
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
      installUpdate().then(() => {
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
    if (e.ctrlKey && key == 'r') {
      e.preventDefault()
      return false
    }

    if (key.match(/^f\d+$/) || key == 'escape') {
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
              <v-icon size="x-large">
                <svg viewBox="0 0 1024 1024" xmlns="http://www.w3.org/2000/svg" width="30" height="30">
                  <path
                      d="M474.112 464a58.432 58.432 0 1 1-58.464-58.4 58.432 58.432 0 0 1 58.464 58.4zM549.632 464A58.432 58.432 0 1 0 608 405.632a58.432 58.432 0 0 0-58.368 58.368z"
                      fill="#1296db" p-id="5837"></path>
                  <path
                      d="M947.2 527.424a152.8 152.8 0 0 1-12.8 0.512 168.256 168.256 0 0 1-74.144-17.312 686.176 686.176 0 0 0 9.984-131.2 681.344 681.344 0 0 0-84.896-100.608 168.768 168.768 0 0 1 59.84-64l10.976-6.816-8.544-9.6a446.88 446.88 0 0 0-156.16-113.92l-11.872-5.184-3.008 12.544a168.16 168.16 0 0 1-42.336 76.8A678.4 678.4 0 0 0 512 118.208a679.04 679.04 0 0 0-122.144 50.304 168.128 168.128 0 0 1-42.208-76.8l-3.04-12.544-11.84 5.152A451.424 451.424 0 0 0 176.48 198.4l-8.576 9.6 10.944 6.72A168.448 168.448 0 0 1 238.56 278.4a684.288 684.288 0 0 0-84.736 100.224 686.624 686.624 0 0 0 9.6 132.096 167.904 167.904 0 0 1-73.6 17.12c-4.544 0-8.8-0.16-12.8-0.512L64 526.432l1.216 12.8A444.416 444.416 0 0 0 125.248 723.2l6.4 11.104 9.824-8.352a168.128 168.128 0 0 1 79.584-37.28 682.016 682.016 0 0 0 68.096 110.944 689.088 689.088 0 0 0 129.088 31.712 167.776 167.776 0 0 1-10.752 88.096l-4.896 11.936 12.608 2.784a451.392 451.392 0 0 0 96.8 10.656l96.672-10.72 12.608-2.784-4.928-11.968a168.224 168.224 0 0 1-10.72-88.128 688.16 688.16 0 0 0 128.576-31.648 682.88 682.88 0 0 0 68.192-111.04 168.896 168.896 0 0 1 80 37.312l9.824 8.32 6.4-11.104a442.784 442.784 0 0 0 60-183.808l1.216-12.8z m-297.152 157.152a521.12 521.12 0 0 1-276.832 0 536.384 536.384 0 0 1-59.264-124.8 530.08 530.08 0 0 1-24.96-136.96 527.488 527.488 0 0 1 100.32-95.52A534.176 534.176 0 0 1 512 260.672a536.352 536.352 0 0 1 122.144 66.4 530.56 530.56 0 0 1 100.768 96 531.2 531.2 0 0 1-25.216 136.352 534.688 534.688 0 0 1-59.488 125.12z"
                      fill="#1296db"></path>
                </svg>
              </v-icon>
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
        :z-index="2000"
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
