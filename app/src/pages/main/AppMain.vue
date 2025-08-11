<script setup lang="ts">

import Home from "~/pages/main/Home.vue";
import Connection from "~/pages/main/Connection.vue";
import {
  _alert,
  _confirm,
  _confirmSystem,
  _listenLocal,
  _loading,
  _unListenLocal,
  EventName,
  SessionDisconnectedEvent
} from "~/common/events.ts";
import {_disconnect} from "~/common/services.ts";
import {onMounted, onUnmounted, reactive, ref} from "vue";
import {SessionData} from "~/common/transport/connection.ts";
import {appWindow, PhysicalSize} from "@tauri-apps/api/window";
import {_exitApp, _isMac, _openMainWindow, _updateMaximizeState} from "~/common/windows.ts";
import {_debounce} from "~/common/utils.ts";
import {_saveGlobalStore, _saveSettings, _useGlobalStore, _useSettings} from "~/common/store.ts";
import {listen} from "@tauri-apps/api/event";
import {MAIN_WINDOW_MIN_HEIGHT, MAIN_WINDOW_MIN_WIDTH, SettingConfig} from "~/common/transport/setting.ts";
import {loadModule, trackEvent} from "~/common/analytics.ts";
import {Handler} from "mitt";
import {useI18n} from "vue-i18n";

type TabItem = {
  name: string,
  session: SessionData,
  hasDirtyContent: boolean,
}

const {t} = useI18n()
const HOME_TAB = "___home"
const activeTab = ref<string>(HOME_TAB)
const tabList = reactive<TabItem[]>([])
const exitConfirmState = ref<boolean>(false)

const eventUnListens = reactive<Function[]>([])

const lastWindowSize = reactive({
  width: 0,
  height: 0
})
const connectorEventBuffer = reactive<Set<Number>>(new Set())

onMounted(async () => {
  try {
    loadModule(true).then(() => {
      trackEvent("create_window_main")
    }).catch(e => {
      console.warn("Failed to load umami script:", e)
    })
  } catch (e) {
    console.error(e)
  }

  _updateMaximizeState()
  let size = await appWindow.innerSize() as PhysicalSize
  lastWindowSize.width = size.width
  lastWindowSize.height = size.height

  //  实时更新最大屏幕状态
  eventUnListens.push(await appWindow.listen('tauri://resize', () => {
    _updateMaximizeState()
  }))
  //  计算窗口大小并写入文件（防抖）
  eventUnListens.push(await appWindow.listen('tauri://resize', _debounce((e: any) => {
    let payload = e.payload as Record<string, number>
    let height = payload.height
    let width = payload.width

    if (width < MAIN_WINDOW_MIN_WIDTH || height < MAIN_WINDOW_MIN_HEIGHT) {
      return
    }

    if (width == lastWindowSize.width && height == lastWindowSize.height) {
      return
    }

    let store = _useGlobalStore().value
    let p1 = appWindow.isFullscreen()
    let p2 = appWindow.isMaximized()
    Promise.all([p1, p2]).then(res => {
      store.windowInitState = {
        mainWindowWidth: width,
        mainWindowHeight: height,
        mainWindowFullscreen: res[0],
        mainWindowMaximize: res[1]
      }

      _saveGlobalStore(store)
      lastWindowSize.width = width
      lastWindowSize.height = height
    }).catch(e => {
      console.error(e)
    })
  }, 1000)))

  eventUnListens.push(await listen(EventName.SETTING_UPDATE, (e) => {
    let setting = JSON.parse(e.payload as string) as SettingConfig
    _saveSettings(setting)
  }))

  eventUnListens.push(await listen(EventName.CONFIRM_EXIT, () => {
    if (exitConfirmState.value) {
      return
    }

    const dirtyTab = findDirtyTab()
    let message = dirtyTab
        ? t('main.home.exitWithDirtyTabConfirm')
        : t('main.home.exitConfirm')
    if (dirtyTab) {
      activeTab.value = dirtyTab.name
    }

    exitConfirmState.value = true
    _confirm(t('main.home.exitConfirmTitle'), message, 20000).then(() => {
      _loading(true, t('main.home.exiting'))
      trackEvent('exit').finally(() => {
        _exitApp().finally(() => {
          _loading(false)
        })
      })
    }).catch(() => {
    }).finally(() => {
      exitConfirmState.value = false
    })
    appWindow.show()
  }))

  eventUnListens.push(await listen<SessionDisconnectedEvent>(EventName.SESSION_DISCONNECTED, event => {
    const sessionId = event.payload.sessionId
    if (connectorEventBuffer.has(sessionId) || findSession(sessionId) < 0) {
      return
    }
    connectorEventBuffer.add(sessionId)
    let message
    if (typeof event.payload.case === "string") {
      switch (event.payload.case) {
        case "sshChannelEof":
          message = 'ssh channel eof'
          break
        case 'sshChannelFailure':
          message = 'ssh channel failure'
          break
        default:
          message = event.payload.case
      }
    } else {
      message = JSON.stringify(event.payload.case)
    }

    _alert(t('main.home.sessionConnLostTip', {reason: message}), t('common.warning')).then(() => {
      connectorEventBuffer.delete(sessionId)
      closeTabDirectly(sessionId)
    })
  }))

  const newConnectionEventHandler: Handler<any> = (e: any) => {
    let name = e.name as string
    let session = e.session as SessionData

    for (let i = tabList.length - 1; i >= 0; i--) {
      let tab = tabList[i]
      if (tab.name == name) {
        name += '(1)'
        break
      }
      if (tab.name.startsWith(name) && tab.name.endsWith(")")) {
        let num = parseInt(tab.name.substring(tab.name.lastIndexOf("(") + 1, tab.name.length))
        name += `(${num + 1})`
        break
      }
    }
    let tabItem: TabItem = {
      name,
      session,
      hasDirtyContent: false
    }
    tabList.push(tabItem)

    activeTab.value = tabItem.name
  }
  _listenLocal(EventName.NEW_CONNECTION, newConnectionEventHandler)
  eventUnListens.push(() => {
    _unListenLocal(EventName.NEW_CONNECTION, newConnectionEventHandler)
  })

  document.addEventListener('keydown', e => {
    let key = e.key.toLowerCase()
    let isMac = _isMac()
    let ctrlKey = (!isMac && e.ctrlKey) || (isMac && e.metaKey)

    //  ctrl+w
    if (ctrlKey && key == 'w') {

      if (_useSettings().value.closeTabUseCtrlW) {
        const currentSession = findSession()
        if (currentSession >= 0 && tabList[currentSession].hasDirtyContent) {
          _confirmSystem(t('main.home.closeDirtyTabConfirm')).then(() => {
            closeTabDirectly()
          }).catch(() => {
          })
        } else {
          closeTabDirectly()
        }
      }

    }
  }, {capture: true})
  const closeTabEventHandler: Handler<any> = e => {
    closeTabDirectly(e as number)
    activeTab.value = HOME_TAB
  }
  _listenLocal(EventName.CLOSE_TAB, closeTabEventHandler)
  eventUnListens.push(() => _unListenLocal(EventName.CLOSE_TAB, closeTabEventHandler))

  _openMainWindow()
})

onUnmounted(() => {
  for (let eventUnListen of eventUnListens) {
    eventUnListen()
  }
})

const findDirtyTab = (id?: number): TabItem | void => {
  for (const tab of tabList) {
    if ((!id || tab.session.id == id) && tab.hasDirtyContent) {
      return tab
    }
  }
}

const closeTab = (id: number) => {
  const dirtyTab = findDirtyTab(id)
  if (dirtyTab) {
    _confirmSystem(t('main.home.closeDirtyTabConfirm')).then(() => {
      closeTabDirectly(id)
    }).catch(() => {
    })
  } else {
    _confirmSystem(t('main.home.closeTabConfirm')).then(() => {
      closeTabDirectly(id)
    }).catch(() => {
    })
  }
}

const findSession = (sessionId?: number): number => {
  let idx = -1;
  let currentTab = activeTab.value
  if (sessionId == undefined && currentTab == HOME_TAB) {
    return idx
  }
  for (let i = 0; i < tabList.length; i++) {
    let item: TabItem = tabList[i]
    if (sessionId == undefined && item.name == currentTab) {
      sessionId = item.session.id
      idx = i;
      break
    }
    if (item.session.id == sessionId) {
      idx = i;
      break
    }
  }
  return idx
}

/**
 * 不许确认关闭连接
 * @param sessionId 连接ID，如果为 undefined 则表示关闭当前tab
 */
const closeTabDirectly = (sessionId?: number) => {
  let idx = findSession(sessionId)

  if (sessionId) {
    _disconnect(sessionId)
  }

  if (idx >= 0) {
    let nextTab = HOME_TAB
    if (tabList.length > 1) {
      if (idx == 0) {
        let next = tabList[idx + 1];
        nextTab = next.name
      } else {
        let next = tabList[idx - 1]
        nextTab = next.name
      }
    }
    activeTab.value = nextTab

    tabList.splice(idx, 1)
  }
}

</script>

<template>
  <div class="fill-height">
    <v-tabs v-model="activeTab"
            show-arrows
            :height="30"
            density="compact"
            color="primary"
    >
      <v-tab icon="mdi-home"
             :value="HOME_TAB"
             density="compact"
             :ripple="false"
             :min-width="50"
      >
        <v-icon>mdi-home</v-icon>
      </v-tab>
      <v-tab v-for="tab in tabList"
             :key="tab.name"
             :value="tab.name"
             class="text-none"
             :ripple="false"
             @click="activeTab = tab.name"
             prepend-icon="mdi-lan-connect"
      >
        <strong v-show="tab.hasDirtyContent">*</strong>
        {{ tab.name }}
        <template v-slot:append>
          <v-icon
              class="tab-icon-close"
              @click="closeTab(tab.session.id)"
              icon="mdi-close"
          />
        </template>
      </v-tab>
    </v-tabs>
    <v-divider/>
    <div style="height: calc(100% - 30px);">
      <Home v-show="activeTab == HOME_TAB"/>
      <Connection :session="tab.session"
                  v-for="tab in tabList"
                  :key="tab.name"
                  v-show="activeTab == tab.name"
                  v-model:has-dirty-content="tab.hasDirtyContent"
      />
    </div>
  </div>
</template>

<style scoped lang="scss">

</style>
