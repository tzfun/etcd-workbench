<script lang="ts" setup>
import {isDark, toggleDark} from "~/composables";
import {reactive, ref} from 'vue'
import type {TabPaneName} from 'element-plus'
import {_checkLogin, _closeSession} from "~/common/Service";
import {EventListener, pushEvent, registerEventListener} from "~/common/Event";
import {unregisterConfigListener} from "~/common/Config";
import {clearLoginStatus, getBuildHash, getUser, getVersion, saveVersionInfo} from "~/common/Store";
import {_nonEmpty} from "~/common/Util";
import {Moon, Sunny, InfoFilled} from "@element-plus/icons-vue";
import etcd from "~/assets/etcd.png";
import {ServerInfo} from "~/common/Types";
import WorkbenchLogo from "~/design/WorkbenchLogo.vue";

let tabIndex = 1
const curTab = ref('1')
const needLogin = ref(false)
const status = ref<'login' | 'main' | 'none'>('none')
const eventListener = ref<EventListener>()
const user = ref()
const etcdLogo = ref(etcd)
const enableInfoDialog = ref(false)
const workbenchInfo = reactive<ServerInfo>({
  version: '',
  buildHash: null
})

onBeforeMount(async () => {
  const result:ServerInfo = await _checkLogin()
  if (result.enableAuth) {
    needLogin.value = true
    if(result.needLogin) {
      status.value = 'login'
      clearLoginStatus()
    } else {
      status.value = 'main'
    }
  } else {
    clearLoginStatus()
    status.value = 'main'
  }
  saveVersionInfo(result.version, result.buildHash)
})

onMounted(() => {
  user.value = getUser()
  eventListener.value = (key: string, event: any): any => {
    if (key === 'login' || key === 'logout') {
      clearLoginStatus()
      status.value = 'login'

      tabs.value.splice(0, tabs.value.length)
      curTab.value = '1'
      tabIndex = 0
      tabAdd()
    } else if (key === 'loginSuccess') {
      status.value = 'main'
    } else if (key == 'storeChange') {
      user.value = getUser()
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
      _closeSession(tab.sessionKey)
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

const handleSelectHeader = (key: string) => {
  if (key == 'logout') {
    pushEvent("logout")
  } else if (key == 'login') {
    pushEvent("login")
  } else if (key == 'github') {
    window.open('https://www.github.com/tzfun/etcd-workbench', '_blank')
  }
}

const showInfo = () => {
  workbenchInfo.version = getVersion()
  workbenchInfo.buildHash = getBuildHash()
  enableInfoDialog.value = true
}
</script>

<template>
  <div style="height: 100%; width: 100%;" class="base-body">
    <div class="header">
      <el-menu
          menu-trigger="click"
          class="header-menu"
          mode="horizontal"
          :ellipsis="false"
          @select="handleSelectHeader"
      >
        <div class="header-item" style="margin-left: 1em">
          <el-image style="width: 30px; height: 30px" :src="etcdLogo" fit="cover"/>
        </div>
        <span class="header-title">ETCD Workbench</span>
        <div class="flex-grow"/>

        <div class="header-item">
          <el-icon class="workbench-info" size="26" @click="showInfo"><InfoFilled /></el-icon>
        </div>
        <div class="header-item">
          <a href="https://www.github.com/tzfun/etcd-workbench" target="_blank" title="Fork from GitHub">
            <svg t="1702187888545"
                 class="icon"
                 viewBox="0 0 1024 1024"
                 version="1.1"
                 xmlns="http://www.w3.org/2000/svg"
                 p-id="8271"
                 width="25"
                 height="25">
              <path
                  d="M511.6 76.3C264.3 76.2 64 276.4 64 523.5 64 718.9 189.3 885 363.8 946c23.5 5.9 19.9-10.8 19.9-22.2v-77.5c-135.7 15.9-141.2-73.9-150.3-88.9C215 726 171.5 718 184.5 703c30.9-15.9 62.4 4 98.9 57.9 26.4 39.1 77.9 32.5 104 26 5.7-23.5 17.9-44.5 34.7-60.8-140.6-25.2-199.2-111-199.2-213 0-49.5 16.3-95 48.3-131.7-20.4-60.5 1.9-112.3 4.9-120 58.1-5.2 118.5 41.6 123.2 45.3 33-8.9 70.7-13.6 112.9-13.6 42.4 0 80.2 4.9 113.5 13.9 11.3-8.6 67.3-48.8 121.3-43.9 2.9 7.7 24.7 58.3 5.5 118 32.4 36.8 48.9 82.7 48.9 132.3 0 102.2-59 188.1-200 212.9 23.5 23.2 38.1 55.4 38.1 91v112.5c0.8 9 0 17.9 15 17.9 177.1-59.7 304.6-227 304.6-424.1 0-247.2-200.4-447.3-447.5-447.3z"
                  p-id="8272"
                  :fill="isDark ? 'white' : 'black'"/>
            </svg>
          </a>
        </div>
        <div class="header-item">
          <el-switch
              title="Theme Switch"
              size="large"
              v-model="isDark"
              :active-action-icon="Moon"
              :inactive-action-icon="Sunny"
          />
        </div>
        <!--        <button class="border-none bg-transparent cursor-pointer header-icon"-->
        <!--            @click="toggleDark()">-->
        <!--          <i inline-flex i="dark:ep-moon ep-sunny"/>-->
        <!--        </button>-->
        <div v-if="needLogin">
          <el-sub-menu index="user" v-if="_nonEmpty(user)">
            <template #title>{{ user }}</template>
            <el-menu-item index="logout">Sign out</el-menu-item>
          </el-sub-menu>
          <el-menu-item index="login" v-else>
            Sign in
          </el-menu-item>
        </div>

      </el-menu>

    </div>

    <Login v-if="status == 'login'"/>
    <div v-else-if="status == 'main'" class="main">
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

    <el-dialog
        v-model="enableInfoDialog"
        title="About"
        width="600"
    >
      <div class="workbench-info-container">
        <workbench-logo/>
        <p class="description">A beautiful and lightweight ETCD V3 client for web</p>
        <p class="copyright">
          Copyright &copy; 2024 <a class="link" target="_blank" href="https://github.com/tzfun">beifengtz</a>. All rights reserved.
        </p>
        <div class="info">
          <div class="info-item">
            <div class="info-label">Version</div>
            <div class="info-content"><strong>{{ workbenchInfo.version }}</strong></div>
          </div>
          <div class="info-item">
            <div class="info-label">Build Hash</div>
            <div class="info-content"><strong>{{ workbenchInfo.buildHash }}</strong></div>
          </div>
          <div class="info-item">
            <div class="info-label">Open Source</div>
            <div class="info-content">
              <a class="link" href="https://github.com/tzfun/etcd-workbench/" target="_blank">Github</a> and
              <a class="link" href="https://gitee.com/tzfun/etcd-workbench/" target="_blank">Gitee</a>
            </div>
          </div>
          <div class="info-item">
            <div class="info-label">LICENSE</div>
            <div class="info-content">
              <a class="link" href="https://github.com/tzfun/etcd-workbench/blob/master/LICENSE" target="_blank">Apache License 2.0</a>
            </div>
          </div>
          <div class="info-item">
            <div class="info-label">Report Bug</div>
            <div class="info-content">
              <a class="link" href="https://github.com/tzfun/etcd-workbench/issues/new" target="_blank">Go to submit</a>
            </div>
          </div>
          <div class="info-item">
            <div class="info-label">Contact Author</div>
            <div class="info-content"><a class="link" href="mailto:beifengtz@qq.com">beifengtz@qq.com</a> </div>
          </div>

        </div>
      </div>

    </el-dialog>
  </div>
</template>

<style lang="scss" scoped>
@import '../styles/index.scss';

.dark {
  .header {
    color: white;
    background: $--header-color-dark;
  }
}

.header {
  height: $--header-height;
  width: 100%;
  color: #494646;
  line-height: $--header-height;
  font-size: 21px;
  font-weight: 800;
  background-color: $--header-color;

  .header-menu {
    --ep-menu-bg-color: $--header-color;
    --ep-menu-item-height: $--header-height;
    height: $--header-height;
  }

  .header-title {
    font-family: system-ui;
    user-select: none;
  }

  .header-icon {
    font-size: 20px;
    margin: 0 5px;
  }

  .header-item {
    display: inline-flex;
    justify-content: center;
    align-items: center;
    height: 100%;
    margin: 0;
    padding: 0 10px;
    border-bottom: 2px solid transparent;
    line-height: var(--ep-menu-item-height);
    font-size: var(--ep-menu-item-font-size);
    box-sizing: border-box;
    white-space: nowrap;

    a {
      display: inline-flex;
    }

    .workbench-info {
      cursor: pointer;
    }
  }
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

.workbench-info-container {

  .description {
    text-align: center;
    font-size: 20px;
  }

  .copyright {
    color: #9f9b9b;
    text-align: center;

    .link {
      color: unset;
      text-decoration-line: none;
      font-weight: 600;
    }

    .link:hover {
      text-decoration-line: underline;
    }
  }

  .info {
    width: max-content;
    text-align: left;
    margin: 50px auto 0 auto;

    .info-item {
      margin: 15px 0;

      .info-label {
        width: 140px;
        text-align: right;
        display: inline-block;
      }

      .info-content {
        display: inline-block;
        margin-left: 10px;

        .link {
          text-decoration-line: none;
        }

        .link:hover {
          text-decoration-line: underline;
        }
      }

      .info-label::after {
        content: ':';
      }
    }
  }
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

<style lang="css">
.base-body .header .ep-switch {
  --ep-switch-on-color: #f2f2f2;
  --ep-switch-off-color: #f2f2f2;
  --ep-switch-border-color: #dcdfe6;
}

.base-body .header .ep-switch__core .ep-switch__action {
  background-color: rgba(0, 0, 0, 0);
  color: black;
}

.dark .base-body .header .ep-switch {
  --ep-switch-on-color: #2c2c2c;
  --ep-switch-off-color: #2c2c2c;
  --ep-switch-border-color: #4c4d4f;
}

.dark .base-body .header .ep-switch__core .ep-switch__action {
  background-color: rgba(0, 0, 0, 0);
  color: white;
}
</style>
