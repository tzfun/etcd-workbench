<script lang="ts" setup>
import {toggleDark} from "~/composables";
import {ref} from 'vue'
import type {TabPaneName} from 'element-plus'
import {closeSession} from "~/service";
import {EventListener, registerEventListener} from "~/util/Event";
import {unregisterConfigListener} from "~/Config";
import {clearLoginStatus} from "~/store";

let tabIndex = 1
const curTab = ref('1')
const showHeader = ref(true)
const status = ref<'login' | 'main'>('main')
const eventListener = ref<EventListener>()

onMounted(() => {
  let params = window.location.search.split("?")[1]
  if (params) {
    let vars = params.split("&")
    for (let v of vars) {
      let kv = v.split("=")
      if (kv[0] == 'header') {
        if (kv[1] === 'hide') {
          showHeader.value = false
        }
        break
      }
    }
  }
  eventListener.value = (key: string, event: any): any => {
    if (key === 'login' || key === 'logout') {
      clearLoginStatus()
      status.value = 'login'
    } else if (key === 'loginSuccess') {
      status.value = 'main'
    }
  }
  registerEventListener(eventListener.value)
})

onUnmounted(() => {
  unregisterConfigListener(eventListener.value)
})

const tabs = ref([
  {
    title: 'New Session(1)',
    name: '1',
    state: 'new',
    sessionKey: undefined
  },
])

const tabAdd = () => {
  let newTabName = ++tabIndex;

  tabs.value.push({
    title: `New Session(${newTabName})`,
    name: newTabName,
    state: 'new',
    sessionKey: undefined
  })
  curTab.value = newTabName
}

const tabRemove = (targetName: TabPaneName) => {
  let idx = -1;
  let tab;
  for (let i = 0; i < tabs.value.length; i++) {
    if (tabs.value[i].name === targetName) {
      idx = i
      tab = tabs.value[i]
      break
    }
  }
  if (idx >= 0) {
    if (tab.state !== 'new') {
      closeSession(tab.sessionKey)
    }
    const nextTab = tabs.value[idx + 1] || tabs.value[idx - 1]
    tabs.value.splice(idx, 1)
    if (nextTab) {
      curTab.value = nextTab.name
    }
  }
  if (tabs.value.length === 0) {
    tabAdd()
  }
}

const onSessionChange = (args: { state: number, name: number, key: string | undefined }, idx: number) => {
  const item = tabs.value[idx]
  item.title = args.name
  item.state = args.state
  item.sessionKey = args.key
}

const checkSessionName = (name: string): boolean => {
  if (name === 'default') {
    return false
  }
  return tabs.value.filter(o => o.title === name).length === 0
}
</script>

<template>
  <div style="height: 100%; width: 100%;">
    <div class="header" v-if="showHeader">
      Etcd Workbench
    </div>
    <button
        class="border-none bg-transparent cursor-pointer light-switch"
        @click="toggleDark()">
      <i inline-flex i="dark:ep-moon ep-sunny"/>
    </button>

    <Login v-if="status == 'login'"/>
    <div v-if="status == 'main'" class="main">
      <el-tabs
          v-model="curTab"
          type="card"
          editable
          class="tabs"
          @tab-add="tabAdd"
          @tab-remove="tabRemove">
        <el-tab-pane
            v-for="(item,idx) in tabs"
            :key="item.name"
            :label="item.title"
            :name="item.name"
            class="tab-pane">
          <EtcdSession @change="onSessionChange($event, idx)" :check-session-name="checkSessionName"/>
        </el-tab-pane>
      </el-tabs>
    </div>

  </div>
</template>

<style lang="scss" scoped>
@import '../styles/index.scss';

.dark {
  .header {
    background: $--header-color-dark;
  }
}

.header {
  height: $--header-height;
  width: 100%;
  color: white;
  text-align: center;
  line-height: $--header-height;
  font-size: 21px;
  font-weight: 800;
  background: $--header-color;
}

.main {
  height: calc(100% - $--header-height - $--footer-height);

  .tabs {
    .tab-pane {
      width: 100%;
      position: fixed;
      height: calc(100% - var(--ep-tabs-header-height) - $--header-height);
      overflow: auto;
    }
  }
}

.light-switch {
  position: fixed;
  width: 50px;
  height: 50px;
  left: 15px;
  bottom: 15px;
  z-index: 10000;
  font-size: 25px;
}
</style>

<style lang="scss">
@import '../styles/index.scss';


.tabs {
  .ep-tabs__header {
    margin: 0;
  }

  .ep-tabs__content {
    //height: calc(100% - var(--ep-tabs-header-height) - $--header-height - 16px);
  }
}
</style>
