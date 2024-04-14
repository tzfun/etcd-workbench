<script lang="ts" setup>

import {ref} from "vue";
import {_heartBeat} from "~/common/Service";
import {ElMessage} from "element-plus";
import {_nonEmpty} from "~/common/Util";
import {SessionDTO, SessionStoreConfig} from "~/common/Types";
import {CirclePlus, Close, Connection} from "@element-plus/icons-vue";
import {
  deleteConf,
  getAllConf,
  loadConfAsync,
  registerConfigListener,
  saveConf,
  unregisterConfigListener
} from "~/common/Config";

const emits = defineEmits(['change'])
defineProps({
  checkSessionName: Function
})
const connectorRef = ref()
const state = ref('new')
const sessionKey = ref<string | undefined>()
const isRoot = ref<boolean>(false)
const heartBeatId = ref()
const configList = ref<SessionStoreConfig[]>([])
const configListener = ref<Function>()

const activeMenu = ref('default')

onMounted(() => {
  try {
    configListener.value = () => {
      configList.value = getAllConf()
    }
    registerConfigListener(configListener.value)
    loadConfAsync()
  } catch (e) {
  }
})

onUnmounted(() => {
  if (heartBeatId.value) {
    clearInterval(heartBeatId.value)
  }
  unregisterConfigListener(configListener.value)
})

const onNewSession = ({sessionInfo, name}) => {
  const key = (sessionInfo as SessionDTO).sessionId

  isRoot.value = (sessionInfo as SessionDTO).root
  sessionKey.value = key
  state.value = 'connected'
  emits('change', {
    state: state.value,
    name: name,
    key: key
  })
  heartBeatId.value = setInterval(() => {
    if (state.value != 'new') {
      _heartBeat(key).catch(e => {
        if (state.value !== 'new') {
          onSessionClosed()
          if (_nonEmpty(e)) {
            ElMessage({
              showClose: true,
              message: e,
              type: "error",
              duration: 3000,
            })
          }
        }
      })
    }
  }, 15000)
}

const onSaveSession = (config: SessionStoreConfig) => {
  saveConf(config)
}

const removeSessionConf = (key: string) => {
  deleteConf(key)
}

const onSessionClosed = () => {
  clearInterval(heartBeatId.value)
  console.debug("Session closed", sessionKey.value)
  state.value = 'new'
  emits('change', {state: state.value, name: "New Session"})
  sessionKey.value = undefined
}

const handleSelectMenu = (key: string) => {
  if (key === 'default') {
    connectorRef.value.resetSessionConfig()
  } else {
    let config = configList.value[parseInt(key)]
    if (config) {
      connectorRef.value.loadSessionConfig(config)
    }
  }
}

</script>

<template>
  <div v-if="state === 'new'" class="connector-container">
    <div class="aside">
      <el-menu
          :default-active="activeMenu"
          class="aside-menu"
          @select="handleSelectMenu">
        <el-menu-item index="default">
          <el-icon>
            <CirclePlus/>
          </el-icon>
          <span>New Session</span>
        </el-menu-item>

        <el-menu-item-group title="Favorites List">
          <el-menu-item v-for="(v,idx) in configList" :key="v.key" :index="idx.toString()">
            <el-icon>
              <Connection/>
            </el-icon>
            <span class="collect-title" :title="v.name">{{ v.name }}</span>
            <el-icon class="aside-menu-close" @click="removeSessionConf(v.key!)" title="remove">
              <Close/>
            </el-icon>
          </el-menu-item>
        </el-menu-item-group>

      </el-menu>
    </div>
    <div class="connector-body">
      <div class="connector">
        <EtcdConnector ref="connectorRef"
                       :check-session-name="checkSessionName"
                       @connected="onNewSession"
                       @save="onSaveSession"/>
      </div>
    </div>
  </div>
  <div v-else-if="state === 'connected'" class="editor">
    <EtcdManager :session-key="sessionKey" :root="isRoot"/>
  </div>
  <div v-else>
    {{ state }}
  </div>
</template>

<style lang="scss" scoped>
@import '../styles/index.scss';

.connector-container {
  width: 100%;
  height: 100%;
  display: flex;
  overflow: hidden;

  .aside {
    width: var(--ep-custom-aside-width);

    .aside-menu {
      width: var(--ep-custom-aside-width);
      height: 100%;
      position: fixed;
      left: 0;
      overflow-y: auto;
      padding-bottom: 100px;

      .collect-title {
        text-overflow: ellipsis;
        max-width: 210px;
        overflow: hidden;
      }

      .aside-menu-close {
        position: absolute;
        right: 0;
        display: inline-block;
        color: #9bcbcb;
      }
    }
  }

  .connector-body {
    width: calc(100% - var(--ep-custom-aside-width));
    overflow: auto;
    padding: 50px;

    .connector {
      display: flex;
      justify-content: center;
    }
  }
}

.editor {
  width: 100%;
  height: 100%;
}
</style>
