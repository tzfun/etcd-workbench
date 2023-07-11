<script lang="ts" setup>

import {ref} from "vue";
import {heartBeat} from "~/services/SessionService";
import {ElMessage} from "element-plus";

const emits = defineEmits(['change'])
defineProps({
  checkSessionName: Function
})
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
          ElMessage({
            showClose: true,
            message: e,
            type: "error",
            duration: 3000,
          })
        }
      })
    }
  }, 15000)
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
  <div v-if="state === 'new'" class="connector">
    <EtcdConnector @connected="onNewSession" :check-session-name="checkSessionName"></EtcdConnector>
  </div>
  <div v-else-if="state === 'connected'" class="editor">
    <EtcdManager :session-key="sessionKey"></EtcdManager>
  </div>
  <div v-else>
    {{ state }}
  </div>
</template>

<style scoped>
.connector {
  display: flex;
  justify-content: center;
}

.editor {
  width: 100%;
  height: 100%;
}
</style>
