<script lang="ts" setup>

import {computed, ref} from "vue";
import {heartBeat} from "~/services/SessionService";
import {ElMessage} from "element-plus";
import {_nonEmpty} from "~/util/Util";
import {SessionDTO, SessionStoreConfig, SessionStoreConfigDict} from "~/entitys/TransformTypes";
import {CirclePlus, Close, Connection} from "@element-plus/icons-vue";
import {deleteConf, getAllConf, registerConfigListener, saveConf} from "~/Config";

const emits = defineEmits(['change'])
defineProps({
  checkSessionName: Function
})
const connectorRef = ref()
const state = ref('new')
const sessionKey = ref<string | undefined>()
const isRoot = ref<boolean>(false)
const heartBeatId = ref()
const configDict = ref<SessionStoreConfigDict>()

const activeMenu = ref('default')

onMounted(async () => {
  configDict.value = await getAllConf(true)
  registerConfigListener(async () => {
    configDict.value = await getAllConf(false)
  })
})

const configList = computed((): SessionStoreConfig[] => {
  let list = []
  for (let key in configDict.value) {
    list.push(configDict.value[key])
  }
  return list
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
      heartBeat(key).catch(e => {
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

onUnmounted(() => {
  if (heartBeatId.value) {
    clearInterval(heartBeatId.value)
  }
})

const handleSelectMenu = (key: string) => {
  if (key === 'default') {
    connectorRef.value.resetSessionConfig()
  } else {
    let config = sessions.value[key]
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

        <el-menu-item-group title="Session Storage">
          <el-menu-item v-for="item in configList" :index="item.key">
            <el-icon>
              <Connection/>
            </el-icon>
            <span>{{ item.name }}</span>
            <el-icon class="aside-menu-close" @click="removeSessionConf(item.key!)">
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
    width: $--ep-custom-aside-width;

    .aside-menu {
      width: $--ep-custom-aside-width;
      height: 100%;
      position: fixed;
      left: 0;
      overflow-y: auto;
      padding-bottom: 100px;

      .aside-menu-close {
        position: absolute;
        right: 0;
        display: inline-block;
        color: #9bcbcb;
      }
    }
  }

  .connector-body {
    width: calc(100% - $--ep-custom-aside-width);
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
