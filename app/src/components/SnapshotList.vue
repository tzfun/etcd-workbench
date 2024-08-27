<script setup lang="ts">
import {onMounted, onUnmounted, reactive, ref, watch} from "vue";
import {
  _handleError,
  _maintenanceListSnapshotTask,
  _maintenanceRemoveSnapshotTask,
  _maintenanceStopSnapshotTask
} from "~/common/services.ts";
import {SnapshotInfo, SnapshotState, SnapshotStateEvent} from "~/common/transport/maintenance.ts";
import {listen} from "@tauri-apps/api/event";
import {_confirmSystem, _listenLocal, EventName} from "~/common/events.ts";
import {_openFolder} from "~/common/windows.ts";
import {_byteTextFormat, _nonEmpty, _pointInRect} from "~/common/utils.ts";
import {appWindow} from "@tauri-apps/api/window";
import {VSheet} from "vuetify/components";

const snapshotList = ref<SnapshotInfo[]>([])
const showList = ref<boolean>(false)

const eventUnListens = reactive<Function[]>([])
const listBoxRef = ref()

const emits = defineEmits(['length-changed','show-changed'])

watch(() => snapshotList.value, (v) => {
  emits('length-changed', v.length)
}, {
  deep: true
})

watch(() => showList.value, (v) => {
  emits('show-changed', v)
})

onMounted(async () => {
  _maintenanceListSnapshotTask().then((list) => {
    for (let info of list) {
      info.state.finished = isFinished(info.state)
    }
    list.sort((a, b) => a.id - b.id)
    snapshotList.value = list
  }).catch(e => {
    _handleError({
      e
    })
  })

  _listenLocal(EventName.SNAPSHOT_CREATE, e => {
    console.log("list changed")
    snapshotList.value.push(e as SnapshotInfo)
    showList.value = true
  })

  eventUnListens.push(await listen(EventName.SNAPSHOT_STATE, e => {
    let stateEvent = e.payload as SnapshotStateEvent
    let info: SnapshotInfo | undefined
    for (let i = 0; i < snapshotList.value.length; i++) {
      let item = snapshotList.value[i];
      if (stateEvent.id == item.id) {
        info = item
        break
      }
    }
    if (info) {
      info.state = stateEvent.state

      info.state.finished = isFinished(info.state)
    }
  }))

  eventUnListens.push(await appWindow.listen('tauri://blur', () => {
    showList.value = false
  }))

  document.addEventListener('mousedown', (e: MouseEvent) => {
    if (showList.value) {
      if (listBoxRef.value) {
        let rect = ((listBoxRef.value as VSheet).$el as HTMLElement).getBoundingClientRect()
        if (rect) {
          if (!_pointInRect(e, rect)) {
            showList.value = false
          }
        }
      }
    }
  })

})

onUnmounted(() => {
  for (let eventUnListen of eventUnListens) {
    eventUnListen()
  }
})

const stopTask = (info: SnapshotInfo) => {
  _confirmSystem('Are you sure you want to stop data backup?').then(() => {
    _maintenanceStopSnapshotTask(info.id).then(() => {
      info.state.finished = true
      info.state.errorMsg = "Stopped"
    }).catch(e => {
      _handleError({
        e
      })
    })
  }).catch(() => {
  })
}

const removeTask = (info: SnapshotInfo, idx: number) => {
  _maintenanceRemoveSnapshotTask(info.id).then(() => {
    snapshotList.value.splice(idx, 1)
  }).catch(e => {
    _handleError({
      e
    })
  })
}

const isFinished = (state: SnapshotState): boolean => {
  return _nonEmpty(state.errorMsg) || state.remain == 0
}

const getProgress = (state: SnapshotState): number => {
  let sum = state.remain + state.received
  if (sum <= 0) {
    return 0
  }
  return 100 * state.received / sum
}

const openFolder = (info: SnapshotInfo) => {
  _openFolder(info.folder, info.name).catch(e => {
    _handleError({
      e
    })
  })
}

</script>

<template>
  <div class="position-relative snapshot-list-container">
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
    <v-sheet class="list-box" v-show="showList" ref="listBoxRef">
      <v-card title="Recent Snapshot"
              border
              flat
      >
        <v-divider></v-divider>
        <v-card-text>
          <div v-if="snapshotList.length > 0">
            <div v-for="(info,idx) in snapshotList"
                 :key="idx"
                 class="list-item"
            >
              <div class="list-item-info">
                <div class="list-item-prepend-icon">
                  <v-icon v-if="info.state.errorMsg != undefined"
                          color="red"
                  >mdi-lightbulb-alert-outline
                  </v-icon>
                  <v-icon v-else-if="info.state.remain == 0"
                          color="green"
                  >mdi-check-circle-outline
                  </v-icon>
                  <v-icon v-else
                          color="secondary"
                  >mdi-download
                  </v-icon>
                </div>

                <div class="list-item-title"

                >
                  <p @click="openFolder(info)"
                     :class="info.state.finished ? 'list-item-title-success' : ''"
                     title="Open the file directory"
                  >{{ info.name }}</p>
                  <p class="v-messages">{{ _byteTextFormat(info.state.received) }}</p>
                </div>
                <div class="list-item-append-icon">
                  <v-icon v-if="info.state.finished"
                          @click="removeTask(info, idx)"
                          title="Delete"
                  >mdi-close
                  </v-icon>
                  <v-icon v-else
                          @click="stopTask(info)"
                          title="Stop"
                  >mdi-stop
                  </v-icon>
                </div>
              </div>
              <v-sheet
                  v-if="!info.state.finished"
                  class="d-flex align-center mx-auto"
                  max-width="250"
              >
                <v-progress-linear
                    :model-value="getProgress(info.state)"
                    color="secondary"
                ></v-progress-linear>

                <strong class="ml-2">{{ Math.ceil(getProgress(info.state)) }}%</strong>
              </v-sheet>
              <v-sheet v-if="info.state.errorMsg"
                       class="d-flex align-center mx-auto"
                       max-width="250"
              >
                <p class="text-red">{{ info.state.errorMsg }}</p>
              </v-sheet>

              <v-divider class="mt-2 mb-2"></v-divider>
            </div>
          </div>
          <v-empty-state v-else
                         icon="mdi-package-variant"
                         :size="40"
                         title="No Records"
          ></v-empty-state>
        </v-card-text>
      </v-card>

    </v-sheet>

  </div>
</template>

<style scoped lang="scss">

.snapshot-list-container {
  display: inline-block;

  .list-box {
    width: 400px;
    position: absolute;
    z-index: 1000;
    top: 28px;
    right: 0;
    text-align: start;

    .list-item {
      $--item-icon-width: 40px;

      .list-item-info {
        display: flex;
        overflow: hidden;
        text-wrap: nowrap;
        text-overflow: ellipsis;

        .list-item-prepend-icon,
        .list-item-append-icon {
          width: $--item-icon-width;
          text-align: center;
          font-size: 1rem;
          padding-top: 5px;
        }

        .list-item-append-icon:hover {
          opacity: 0.6;
        }

        .list-item-title {
          width: calc(100% - $--item-icon-width * 2);
          text-align: start;
          padding: 5px 15px;
          vertical-align: middle;
          font-size: 1rem;
          text-overflow: ellipsis;
          white-space: nowrap;
          overflow: hidden;
        }
        .list-item-title-success {
          cursor: pointer;
        }
        .list-item-title-success:hover {
          opacity: 0.7;
        }
      }
    }
  }
}


.v-theme--dark {
  .list-box {
    box-shadow: 5px 5px 30px rgba(0, 0, 0, .7);
    border: solid rgba(33, 33, 33, 0.12) 1px;
  }
}

.v-theme--light {
  .list-box {
    box-shadow: 5px 5px 20px rgba(0, 0, 0, .2);
  }
}
</style>
