<script lang="ts" setup>
import {toggleDark} from "~/composables";
import {ref} from 'vue'
import type {TabPaneName} from 'element-plus'
import {closeSession} from "~/services/SessionService";
import {EventCenter} from "~/util/EventCenter";

let tabIndex = 1
const editableTabsValue = ref('1')
const tabs = ref([
  {
    title: 'New Session',
    name: '1',
    state: 'new',
    sessionKey: undefined
  },
])

const handleTabsEdit = (targetName: TabPaneName | undefined, action: 'remove' | 'add') => {
  if (action === 'add') {
    const newTabName = `${++tabIndex}`
    tabs.value.push({
      title: `New Session(${newTabName})`,
      name: newTabName,
      state: 'new',
      sessionKey: undefined
    })
    editableTabsValue.value = newTabName
  } else if (action === 'remove') {
    const tabsVal = tabs.value
    let activeName = editableTabsValue.value
    if (activeName === targetName) {
      tabsVal.forEach((tab, index) => {
        if (tab.name === targetName) {
          const removeTab = () => {
            const nextTab = tabsVal[index + 1] || tabsVal[index - 1]
            if (nextTab) {
              activeName = nextTab.name
            }
          }
          if (tab.state !== 'new') {
            EventCenter('closeSession', tab.sessionKey)
            closeSession(tab.sessionKey)
          }
          removeTab()
          return
        }
      })
    }

    editableTabsValue.value = activeName
    tabs.value = tabsVal.filter((tab) => tab.name !== targetName)
  }
}

const onSessionChange = (args: { state: number, name: number, key: string | undefined }, idx: number) => {
  console.log(idx, args)
  const item = tabs.value[idx]
  item.title = args.name
  item.state = args.state
  item.sessionKey = args.key
}
</script>

<template>
  <el-tabs
      v-model="editableTabsValue"
      type="card"
      editable
      class="tabs"
      @edit="handleTabsEdit"
  >
    <button
        class="border-none bg-transparent cursor-pointer"
        @click="toggleDark()"
    >
      <i inline-flex i="dark:ep-moon ep-sunny"/>
    </button>
    <el-tab-pane
        v-for="(item,idx) in tabs"
        :key="item.name"
        :label="item.title"
        :name="item.name"
        class="tab-pane"
    >
      <EtcdSession @change="onSessionChange($event, idx)"
                   :check-session-name="name => tabs.filter(o => o.title === name).length === 0"
      ></EtcdSession>
    </el-tab-pane>

  </el-tabs>
</template>

<style scoped>
.tabs, .tab-pane {
  width: 100%;
}
</style>

<style>
.tabs .ep-tabs__content {
  height: calc(100% - var(--ep-tabs-header-height) - 16px);
}
</style>