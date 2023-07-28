<script lang="ts" setup>

import {ref} from "vue";
import {heartBeat} from "~/services/SessionService";
import {ElMessage} from "element-plus";
import {_nonEmpty} from "~/util/Util";
import {SessionStoreConfig} from "~/entitys/TransformTypes";

const emits = defineEmits(['change'])
defineProps({
  checkSessionName: Function
})
const connectorRef = ref()
const state = ref('new')
const sessionKey = ref<string | undefined>()
const heartBeatId = ref()

const onNewSession = ({key, name}) => {
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
  console.log(config)
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

</script>

<template>
  <div v-if="state === 'new'">
    <div class="aside">

    </div>
    <div class="connector">
      <EtcdConnector ref="connectorRef"
                     :check-session-name="checkSessionName"
                     @connected="onNewSession"
                     @save="onSaveSession"/>
    </div>
  </div>
  <div v-else-if="state === 'connected'" class="editor">
    <EtcdManager :session-key="sessionKey"/>
  </div>
  <div v-else>
    {{ state }}
  </div>
</template>

<style lang="scss" scoped>
.connector {
  display: flex;
  justify-content: center;
}

.editor {
  width: 100%;
  height: 100%;
}
</style>
