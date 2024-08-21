<script setup lang="ts">

import {onMounted, onUnmounted, reactive, ref} from "vue";
import {
  _handleError,
  _maintenanceListSnapshotTask,
  _maintenanceRemoveSnapshotTask,
  _maintenanceStopSnapshotTask
} from "~/common/services.ts";
import {SnapshotStateInfo} from "~/common/transport/maintenance.ts";
import {listen} from "@tauri-apps/api/event";
import {_confirm, EventName} from "~/common/events.ts";
import {info} from "sass";

const snapshotList = ref<SnapshotStateInfo[]>([])
const showList = ref<boolean>(false)

const eventUnListens = reactive<Function[]>([])

onMounted(async () => {
  _maintenanceListSnapshotTask().then((list) => {
    snapshotList.value = list
    console.log(list)
  }).catch(e => {
    _handleError({
      e
    })
  })

  eventUnListens.push(await listen(EventName.SNAPSHOT_STATE, e => {
    let info = e.payload as SnapshotStateInfo
    console.log(info)
    for (let i = 0; i < snapshotList.value.length; i++) {
      let item = snapshotList.value[i];
      if (info.id == item.id) {
        snapshotList.value[i] = info
        return
      }
    }
    snapshotList.value.push(info)
  }))
})

onUnmounted(() => {
  for (let eventUnListen of eventUnListens) {
    eventUnListen()
  }
})

const stopTask = (info: SnapshotStateInfo) => {
  _confirm('Are you sure you want to stop data backup?').then(() => {
    _maintenanceStopSnapshotTask(info.id).then(() => {
      info.state.finished = true
      info.state.error_msg = "Stopped"
    }).catch(e => {
      _handleError({
        e
      })
    })
  }).catch(() => {
  })
}

const removeTask = (info: SnapshotStateInfo, idx: number) => {
  _maintenanceRemoveSnapshotTask(info.id).then(() => {
    snapshotList.value.splice(idx, 1)
  }).catch(e => {
    _handleError({
      e
    })
  })
}

</script>

<template>
  <div class="d-inline-block position-relative">
    <v-btn class="system-extend-btn ms-2"
           icon="mdi-cloud-arrow-down"
           size="small"
           variant="text"
           :rounded="false"
           density="comfortable"
           title="Snapshot tasks"
           :ripple="false"
           @click="showList = !showList"
    ></v-btn>
    <v-sheet class="list-box" v-show="showList">
      <v-list v-if="snapshotList.length > 0">
        <v-list-item v-for="(info,idx) in snapshotList"
                     :key="idx"
                     :title="info.name"
                     :value="info.id"
                     :prepend-icon="info.state.finished ? 'mdi-check-circle-outline' : 'mdi-download'"
        >
          <template #append>
            <v-btn v-if="info.state.finished"
                   @click="removeTask(info, idx)"
                   icon="mdi-close"
                   rounded
            ></v-btn>
            <v-btn v-else
                   @click="stopTask(info)"
                   icon="mdi-pause"
                   rounded
            ></v-btn>
          </template>
        </v-list-item>
      </v-list>
      <v-empty-state v-else
                     icon="mdi-package-variant"
                     :size="50"
                     title="No Task"></v-empty-state>
    </v-sheet>

  </div>
</template>

<style scoped lang="scss">

.list-box {
  min-width: 200px;
  min-height: 120px;
  position: absolute;
  z-index: 1000;
  top: 28px;
  right: 0;
}

.v-theme--dark {
  .list-box {
    background-color: #637475;
    box-shadow: 5px 5px 150px rgba(255, 255, 255, .2);
    border: solid rgba(33, 33, 33, 0.12) 1px;
  }
}

.v-theme--light {
  .list-box {
    background-color: #484f50;
  }
}
</style>