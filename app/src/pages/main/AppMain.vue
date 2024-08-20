<script setup lang="ts">

import Home from "~/pages/main/Home.vue";
import Connection from "~/pages/main/Connection.vue";
import {_confirm, localEvents} from "~/common/events.ts";
import {_disconnect} from "~/common/services.ts";
import {computed, onMounted, onUnmounted, reactive, ref} from "vue";
import {SessionData} from "~/common/transport/connection.ts";
import {appWindow, PhysicalSize} from "@tauri-apps/api/window";
import {_openMainWindow} from "~/common/windows.ts";
import {_debounce, fileTypeIcon} from "~/common/utils.ts";
import {_saveSettings, _useSettings} from "~/common/store.ts";
import {listen} from "@tauri-apps/api/event";
import {SettingConfig} from "~/common/transport/setting.ts";

type TabItem = {
  name: string,
  session: SessionData
}

const HOME_TAB = "___home"
const activeTab = ref<string>(HOME_TAB)
const tabList = reactive<TabItem[]>([])

const eventUnListens = reactive<Function[]>([])

const props = defineProps({
  platform: {
    type: String,
    required: true
  }
})

const isWindows = computed<boolean>(() => {
  return props.platform == 'win32'
})

const isMac = computed<boolean>(() => {
  return props.platform == 'darwin'
})

onMounted(async () => {
  eventUnListens.push(await appWindow.listen('tauri://resize', _debounce((e) => {
    let payload = e.payload as Record<string, number>
    let height = payload.height
    let width = payload.width

    let setting = _useSettings().value
    let p1 = appWindow.isFullscreen()
    let p2 = appWindow.isMaximized()
    Promise.all([p1, p2]).then(res => {
      setting.windowInitState = {
        mainWindowWidth: width,
        mainWindowHeight: height,
        mainWindowFullscreen: res[0],
        mainWindowMaximize: res[1]
      }

      _saveSettings(setting)
    }).catch(e => {
      console.error(e)
    })
  }), 1000))

  eventUnListens.push(await listen('settingUpdate', (e) => {
    let setting = JSON.parse(e.payload as string) as SettingConfig

    let p1 = appWindow.isFullscreen()
    let p2 = appWindow.isMaximized()
    let p3 = appWindow.innerSize()

    Promise.all([p1, p2, p3]).then(res => {
      let size = res[2] as PhysicalSize
      setting.windowInitState = {
        mainWindowWidth: size.width,
        mainWindowHeight: size.height,
        mainWindowFullscreen: res[0],
        mainWindowMaximize: res[1]
      }

      console.log("save setting", setting)
      _saveSettings(setting)
    }).catch(e => {
      console.error(e)
    })
  }))

  localEvents.on('newConnection', (e: any) => {
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
    let tabItem = {
      name,
      session
    }
    tabList.push(tabItem)

    activeTab.value = tabItem.name
  })

  document.addEventListener('keydown', e => {
    let key = e.key.toLowerCase()

    let ctrlKey = (isWindows.value && e.ctrlKey) || (isMac.value && e.metaKey)

    //  ctrl+w
    if (ctrlKey && key == 'w') {

      if (_useSettings().value.closeTabUseCtrlW) {
        closeTabDirectly()
      }

    }
  }, {capture: true})

  localEvents.on('closeTab', e => {
    closeTabDirectly(e as number)
    activeTab.value = HOME_TAB
  })

  _openMainWindow()
})

onUnmounted(() => {
  for (let eventUnListen of eventUnListens) {
    eventUnListen()
  }
})

const closeTab = (id: number) => {
  _confirm('System', 'Are you sure to close the current connection?').then(() => {
    closeTabDirectly(id)
  }).catch(() => {
  })
}

/**
 * 不许确认关闭连接
 * @param sessionId 连接ID，如果为 undefined 则表示关闭当前tab
 */
const closeTabDirectly = (sessionId: number | undefined) => {
  let currentTab = activeTab.value
  if (sessionId == undefined && currentTab == HOME_TAB) {
    return
  }
  let idx = -1;
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

  _disconnect(sessionId)

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
        {{ tab.name }}
        <template v-slot:append>
          <v-icon class="tab-icon-close" @click="closeTab(tab.session.id)">mdi-close</v-icon>
        </template>
      </v-tab>
    </v-tabs>
    <v-divider></v-divider>
    <div style="height: calc(100% - 30px);">
      <Home v-show="activeTab == HOME_TAB"></Home>
      <Connection :session="tab.session"
                  v-for="tab in tabList"
                  :key="tab.name"
                  v-show="activeTab == tab.name"
      ></Connection>
    </div>
  </div>
</template>

<style scoped lang="scss">

</style>
