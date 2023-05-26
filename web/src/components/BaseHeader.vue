<script lang="ts" setup>
import {toggleDark} from "~/composables";
import {ref} from 'vue'
import type {TabPaneName} from 'element-plus'

let tabIndex = 2
const editableTabsValue = ref('1')
const tabs = ref([
  {
    title: 'New Session',
    name: '1',
    session: {
      key: ''
    }
  },
])

const handleTabsEdit = (targetName: TabPaneName | undefined, action: 'remove' | 'add') => {
  if (action === 'add') {
    const newTabName = `${++tabIndex}`
    tabs.value.push({
      title: 'New Session',
      name: newTabName,
      session: {
        key: ''
      }
    })
    editableTabsValue.value = newTabName
  } else if (action === 'remove') {
    const tabsVal = tabs.value
    let activeName = editableTabsValue.value
    if (activeName === targetName) {
      tabsVal.forEach((tab, index) => {
        if (tab.name === targetName) {
          const nextTab = tabsVal[index + 1] || tabsVal[index - 1]
          if (nextTab) {
            activeName = nextTab.name
          }
        }
      })
    }

    editableTabsValue.value = activeName
    tabs.value = tabsVal.filter((tab) => tab.name !== targetName)
  }
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
        v-for="item in tabs"
        :key="item.name"
        :label="item.title"
        :name="item.name"
        class="tab-pane"
    >
      <EtcdSession :key="item.session.key"></EtcdSession>
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