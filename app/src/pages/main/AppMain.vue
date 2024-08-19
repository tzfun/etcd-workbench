<script setup lang="ts">

import Home from "~/pages/main/Home.vue";
import Connection from "~/pages/main/Connection.vue";
import {_confirm, localEvents} from "~/common/events.ts";
import {_disconnect} from "~/common/services.ts";
import {onMounted, onUnmounted, reactive, ref} from "vue";
import {SessionData} from "~/common/transport/connection.ts";
import {appWindow} from "@tauri-apps/api/window";
import {_openMainWindow} from "~/common/windows.ts";

type TabItem = {
  name: string,
  session: SessionData
}

const HOME_TAB = "___home"
const activeTab = ref<string>(HOME_TAB)
const tabList = reactive<TabItem[]>([])

const eventUnListens = reactive<Function[]>([])

onMounted(async () => {
  eventUnListens.push(await appWindow.listen('tauri://resize', (e) => {
    let payload = e.payload as Record<string, number>
    let height = payload.height
    let width = payload.width
    console.log("resize", height, width)
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

const closeTabDirectly = (sessionId: number) => {
  let idx = -1;
  for (let i = 0; i < tabList.length; i++) {
    let item: TabItem = tabList[i]
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
