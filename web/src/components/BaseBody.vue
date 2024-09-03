<script lang="ts" setup>
import {isDark} from "~/composables";
import {reactive, ref} from 'vue'
import type {TabPaneName} from 'element-plus'
import {_checkLogin, _closeSession} from "~/common/Service";
import {EventListener, pushEvent, registerEventListener} from "~/common/Event";
import {unregisterConfigListener} from "~/common/Config";
import {clearLoginStatus, getBuildHash, getUser, getVersion, saveInfo} from "~/common/Store";
import {_nonEmpty} from "~/common/Util";
import {Moon, Sunny, InfoFilled} from "@element-plus/icons-vue";
import logo from "~/assets/logo.png";
import {ServerInfo} from "~/common/Types";
import WorkbenchLogo from "~/design/WorkbenchLogo.vue";

let tabIndex = 1
const curTab = ref('1')
const needLogin = ref(false)
const status = ref<'login' | 'main' | 'none'>('none')
const eventListener = ref<EventListener>()
const user = ref()
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
  saveInfo(result.version, result.buildHash, result.enableHeartbeat)
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
        <div class="header-item user-select-none pointer-events-none" style="margin-left: 1em">
          <el-image style="width: 30px; height: 30px" :src="logo" fit="cover"/>
        </div>
        <span class="header-title user-select-none pointer-events-none">ETCD Workbench</span>
        <div class="flex-grow"/>

        <div class="header-item">
          <a href="https://github.com/tzfun/etcd-workbench/releases" target="_blank" title="Download App">
            <svg viewBox="0 0 1024 1024"
                 xmlns="http://www.w3.org/2000/svg"
                 width="25"
                 height="25">
              <path d="M768 42.688a128 128 0 0 1 128 128V256c47.104 0 85.312 38.208 85.312 85.312v256c0 47.168-38.208 85.376-85.312 85.376v170.624a128 128 0 0 1-128 128H256a128 128 0 0 1-128-128v-170.624A85.312 85.312 0 0 1 42.688 597.312v-256C42.688 294.208 80.896 256 128 256V170.688a128 128 0 0 1 128-128h512z m42.688 639.936H213.312v170.688c0 23.104 18.368 42.048 41.472 42.688H768a42.688 42.688 0 0 0 42.688-41.472v-171.904z m-238.976 42.688c28.48 0 42.688 14.272 42.688 42.688 0 28.416-14.208 42.688-42.688 42.688H452.288c-28.48 0-42.688-14.272-42.688-42.688 0-28.416 14.208-42.688 42.688-42.688h119.424zM308.48 353.024h-52.16L161.088 597.312h52.352l20.16-55.488h97.664l21.312 55.488h53.696L308.416 353.024z m203.136 0H432.448v244.288h49.28v-92.16h32.192a281.6 281.6 0 0 0 51.2-3.456c8.64-1.92 17.152-5.76 25.6-11.584 8.32-5.888 15.232-13.888 20.672-24.128 5.44-10.24 8.192-22.848 8.192-37.76 0-19.52-4.736-35.392-14.144-47.616a64.064 64.064 0 0 0-35.2-23.936c-9.088-2.432-28.672-3.648-58.688-3.648z m227.712 0h-79.168v244.288h49.28v-92.16h32.192a281.6 281.6 0 0 0 51.2-3.456c8.64-1.92 17.152-5.76 25.6-11.584 8.32-5.888 15.296-13.888 20.736-24.128 5.44-10.24 8.128-22.848 8.128-37.76 0-19.52-4.736-35.392-14.144-47.616a64.064 64.064 0 0 0-35.2-23.936c-9.088-2.432-28.608-3.648-58.624-3.648z m-457.536 56.96l33.664 90.688H248.768l32.96-90.688zM505.6 394.304c17.792 0 29.568 0.64 35.456 1.728a34.112 34.112 0 0 1 19.84 10.816c5.248 5.76 7.872 13.12 7.872 22.016a32.384 32.384 0 0 1-20.992 30.976c-6.592 2.56-19.584 3.84-39.04 3.84h-27.008V394.24z m227.648 0c17.792 0 29.632 0.64 35.52 1.728a34.112 34.112 0 0 1 19.84 10.816c5.184 5.76 7.808 13.12 7.808 22.016 0 7.168-1.92 13.504-5.568 18.944a32.384 32.384 0 0 1-15.424 12.032c-6.592 2.56-19.584 3.84-39.04 3.84h-26.944V394.24zM769.216 128H256a42.688 42.688 0 0 0-42.688 41.472V256h597.376V170.688A42.688 42.688 0 0 0 769.216 128z"
                    :fill="isDark ? 'white' : 'black'"/>
            </svg>
          </a>
        </div>
        <div class="header-item">
          <el-icon class="workbench-info" size="26" @click="showInfo"><InfoFilled /></el-icon>
        </div>
        <div class="header-item">
          <a href="https://www.github.com/tzfun/etcd-workbench" target="_blank" title="Fork from GitHub">
            <svg viewBox="0 0 1024 1024"
                 xmlns="http://www.w3.org/2000/svg"
                 width="25"
                 height="25">
              <path
                  d="M511.6 76.3C264.3 76.2 64 276.4 64 523.5 64 718.9 189.3 885 363.8 946c23.5 5.9 19.9-10.8 19.9-22.2v-77.5c-135.7 15.9-141.2-73.9-150.3-88.9C215 726 171.5 718 184.5 703c30.9-15.9 62.4 4 98.9 57.9 26.4 39.1 77.9 32.5 104 26 5.7-23.5 17.9-44.5 34.7-60.8-140.6-25.2-199.2-111-199.2-213 0-49.5 16.3-95 48.3-131.7-20.4-60.5 1.9-112.3 4.9-120 58.1-5.2 118.5 41.6 123.2 45.3 33-8.9 70.7-13.6 112.9-13.6 42.4 0 80.2 4.9 113.5 13.9 11.3-8.6 67.3-48.8 121.3-43.9 2.9 7.7 24.7 58.3 5.5 118 32.4 36.8 48.9 82.7 48.9 132.3 0 102.2-59 188.1-200 212.9 23.5 23.2 38.1 55.4 38.1 91v112.5c0.8 9 0 17.9 15 17.9 177.1-59.7 304.6-227 304.6-424.1 0-247.2-200.4-447.3-447.5-447.3z"
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
        append-to-body
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
            <div class="info-label">App</div>
            <div class="info-content">
              <a class="link" href="https://github.com/tzfun/etcd-workbench/releases" target="_blank">Download</a>
              <svg
                  style="margin-left: 5px"
                  viewBox="0 0 1024 1024"
                  xmlns="http://www.w3.org/2000/svg"
                  width="25"
                  height="25">
                <path d="M245.76 286.72h552.96c124.928 0 225.28 100.352 225.28 225.28s-100.352 225.28-225.28 225.28H0V532.48c0-135.168 110.592-245.76 245.76-245.76z m133.12 348.16V401.408H348.16v178.176l-112.64-178.176H204.8V634.88h30.72v-178.176L348.16 634.88h30.72z m182.272-108.544v-24.576h-96.256v-75.776h110.592v-24.576h-141.312V634.88h143.36v-24.576h-112.64v-83.968h96.256z m100.352 28.672l-34.816-151.552h-34.816l55.296 233.472H675.84l47.104-161.792 4.096-20.48 4.096 20.48 47.104 161.792h28.672l57.344-233.472h-34.816l-32.768 151.552-4.096 30.72-6.144-30.72-40.96-151.552h-30.72l-40.96 151.552-6.144 30.72-6.144-30.72z"
                      fill="#EE502F"/>
              </svg>
            </div>
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
