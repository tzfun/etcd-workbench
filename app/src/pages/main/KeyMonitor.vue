<script setup lang="ts">
import {computed, onMounted, onUnmounted, PropType, reactive, ref} from "vue";
import {SessionData, KeyMonitorConfig} from "~/common/transport/connection.ts";
import {_confirmSystem, _emitLocal, EventName, KeyWatchEvent} from "~/common/events.ts";
import {
  _decodeBytesToString,
  _isEmpty,
  _timeFormat,
  _tryParseDiffLanguage,
  _tryParseEditorLanguage
} from "~/common/utils.ts";
import {CodeDiff} from "v-code-diff";
import {useTheme} from "vuetify";
import {_handleError, _removeKeyMonitor, _setKeyMonitor} from "~/common/services.ts";
import {EditorHighlightLanguage} from "~/common/types";
import {KeyValue} from "~/common/transport/kv.ts";

const theme = useTheme()

const valueDiffDialog = reactive({
  show: false,
  key: <string>"",
  beforeValue: <string>"",
  afterValue: <string>"",
  language: <string>"",
  beforeKv: <KeyValue | undefined> undefined,
  afterKv: <KeyValue | undefined> undefined,
})

const props = defineProps({
  session: {
    type: Object as PropType<SessionData>,
    required: true
  },
  events: {
    type: Array<KeyWatchEvent>,
    required: true,
  }
})
const monitorListDialog = ref(false)
const searchKeyword = ref<string>("");
const isDarkTheme = computed<boolean>(() => {
  return theme.global.name.value === 'dark'
})

const emits = defineEmits(['on-read'])
const eventUnListens = reactive<Function[]>([])
const monitorList = computed<KeyMonitorConfig[]>(() => {
  if (props.session!.keyMonitorMap) {
    let list = Object.values(props.session!.keyMonitorMap)

    list = list.sort((a, b) => a.key.localeCompare(b.key))
    if (!_isEmpty(searchKeyword.value)) {
      list = list.filter(monitor => monitor.key.toLowerCase().includes(searchKeyword.value.toLowerCase()))
    }
    return list
  }
  return []
})
onMounted(() => {

})

onUnmounted(() => {
  for (let eventUnListen of eventUnListens) {
    eventUnListen()
  }
})

const markAllRead = () => {
  for (let e of props.events) {
    e.read = true
  }
  emits('on-read', -1)
}

const read = (e: KeyWatchEvent) => {
  if (!e.read) {
    e.read = true
    emits('on-read', 1)
  }

  if (e.eventType == 'Modify') {
    valueDiffDialog.key = e.eventKey!

    let editorLang;
    if (e.prevKv && e.curKv) {
      valueDiffDialog.beforeKv = e.prevKv
      valueDiffDialog.afterKv = e.curKv

      if (e.prevKv.formattedValue && e.curKv.formattedValue) {
        valueDiffDialog.beforeValue = e.prevKv.formattedValue.value
        valueDiffDialog.afterValue = e.curKv.formattedValue.value
        editorLang = e.prevKv.formattedValue.language as EditorHighlightLanguage
      } else {
        valueDiffDialog.beforeValue = _decodeBytesToString(e.prevKv.value)
        valueDiffDialog.afterValue = _decodeBytesToString(e.curKv.value)
        let validContent = _isEmpty(valueDiffDialog.beforeValue) ? valueDiffDialog.afterValue : valueDiffDialog.beforeValue
        editorLang = _tryParseEditorLanguage(e.key, validContent, undefined, props.session?.namespace)
      }

      valueDiffDialog.language = _tryParseDiffLanguage(editorLang)
      valueDiffDialog.show = true
    }
  }
}

const clearHistory = () => {
  _confirmSystem("Are you sure you want to clear all history?").then(() => {
    let len = props.events?.length
    if (len > 0) {
      props.events?.splice(0, len)
    }
    markAllRead()
  }).catch(() => {
  })
}

const removeMonitor = (key: string) => {
  _removeKeyMonitor(props.session?.id, key).then(() => {
    delete props.session!.keyMonitorMap![key]
    _emitLocal(EventName.KEY_MONITOR_CONFIG_CHANGE, {
      session: props.session?.id,
      key: key,
      type: 'remove'
    })
  }).catch((e) => {
    _handleError({
      e,
      session: props.session
    })
  })
}

const editKeyMonitor = (key: string) => {
  let monitor: KeyMonitorConfig = props.session?.keyMonitorMap![key]

  if (monitor) {
    _emitLocal(EventName.EDIT_KEY_MONITOR, {
      session: props.session?.id,
      edit: true,
      monitor
    })
  }
}

const pauseMonitor = (config: KeyMonitorConfig, paused: boolean) => {
  config.paused = paused
  _setKeyMonitor(props.session?.id, config).then(() => {
    props.session!.keyMonitorMap![config.key] = config
    _emitLocal(EventName.KEY_MONITOR_CONFIG_CHANGE, {
      session: props.session?.id,
      key: config.key,
      type: 'create',
      config
    })
  }).catch(e => {
    _handleError({
      e,
      session: props.session
    })
  })
}

const addMonitor = () => {
  _emitLocal(EventName.EDIT_KEY_MONITOR, {
    session: props.session?.id,
    edit: false
  })
}

</script>

<template>
  <div class="fill-height pa-5 overflow-y-auto">
    <v-layout>
      <v-btn class="text-none"
             prepend-icon="mdi-checkbox-marked-circle-auto-outline"
             :disabled="events.length == 0"
             @click="markAllRead"
             color="primary"
      >Mark All Read
      </v-btn>
      <v-btn class="text-none ml-2"
             prepend-icon="mdi-delete-circle-outline"
             :disabled="events.length == 0"
             @click="clearHistory"
             color="red"
      >Clear History
      </v-btn>
      <v-btn class="text-none ml-2"
             prepend-icon="mdi-robot"
             @click="monitorListDialog = true"
             color="#cc8f53"
      >My Monitors
      </v-btn>

      <v-spacer/>

    </v-layout>
    <div style="height: calc(100% - 56px); overflow-y: auto;">
      <v-list class="pa-0 my-5 overflow-hidden"
              :selectable="false"
              lines="two"
              v-if="events.length > 0"
      >
        <transition-group name="event-list">
          <v-list-item
              v-for="e in events"
              :key="e.id"
              :title="e.eventKey"
              :subtitle="_timeFormat(e.eventTime)"
              @click="read(e)"
              density="comfortable"
              :variant="e.read ? 'plain' : 'tonal'"
          >
            <template #prepend>
              <v-icon
                  v-if="e.eventType == 'Create'"
                  color="light-blue-darken-1"
              >mdi-folder-plus-outline</v-icon>
              <v-icon
                  v-else-if="e.eventType == 'Remove'"
                  color="red-darken-2"
              >mdi-folder-remove-outline</v-icon>
              <v-icon
                  v-else-if="e.eventType == 'Modify'"
                  color="yellow-lighten-1"
              >mdi-content-save-all-outline</v-icon>
            </template>

            <template #append>
              <span v-if="e.eventType == 'Create'" class="text-medium-emphasis">Created</span>
              <span v-else-if="e.eventType == 'Remove'" class="text-medium-emphasis">Removed</span>
              <span v-else-if="e.eventType == 'Modify'" class="text-medium-emphasis">Value Changed</span>

              <v-tooltip location="end center"
                         origin="start center"
                         no-click-animation
                         :text="`From monitor: ${e.key}`">
                <template v-slot:activator="{ props }">
                  <v-icon color="green"
                          v-bind="props"
                          class="ml-2"
                          size="small"
                  >mdi-robot
                  </v-icon>
                </template>
              </v-tooltip>
            </template>
          </v-list-item>
        </transition-group>
      </v-list>

      <v-empty-state v-else
                     icon="mdi-package-variant"
                     headline="No Notification"
                     class="user-select-none"
      ></v-empty-state>
    </div>

    <!--    Diff弹窗  -->
    <v-dialog
        v-model="valueDiffDialog.show"
        max-width="80vw"
        min-width="500px"
        scrollable
    >
      <v-card
          :min-width="800"
          :title="valueDiffDialog.key"
          prepend-icon="mdi-content-save-all-outline"
      >
        <template v-slot:append>
          <v-icon class="cursor-pointer" @click="valueDiffDialog.show = false">mdi-close</v-icon>
        </template>
        <v-card-text>
          <v-layout class="diff-kv-info">
            <div>{{ valueDiffDialog.beforeKv!.lease }}</div>
            <v-spacer></v-spacer>
            <span class="text-medium-emphasis">Lease</span>
            <v-spacer></v-spacer>
            <div>{{ valueDiffDialog.afterKv!.lease }}</div>
          </v-layout>
          <v-layout class="diff-kv-info">
            <div>{{ valueDiffDialog.beforeKv!.version }}</div>
            <v-spacer></v-spacer>
            <span class="text-medium-emphasis">Version</span>
            <v-spacer></v-spacer>
            <div>{{ valueDiffDialog.afterKv!.version }}</div>
          </v-layout>
          <v-layout class="diff-kv-info">
            <div>{{ valueDiffDialog.beforeKv!.modRevision }}</div>
            <v-spacer></v-spacer>
            <span class="text-medium-emphasis">Modify Revision</span>
            <v-spacer></v-spacer>
            <div>{{ valueDiffDialog.afterKv!.modRevision }}</div>
          </v-layout>

          <code-diff
              style="max-height: 60vh;min-height: 40vh;"
              :old-string="valueDiffDialog.beforeValue"
              filename="Before"
              :new-string="valueDiffDialog.afterValue"
              new-filename="After"
              :theme="isDarkTheme ? 'dark' : 'light'"
              :language="valueDiffDialog.language"
              output-format="side-by-side"/>
        </v-card-text>
      </v-card>
    </v-dialog>

    <!--   key monitor-->
    <v-dialog
        v-model="monitorListDialog"
        eager
        transition="slide-y-reverse-transition"
        scrollable
        class="collection-drawer-bottom"
        contained
    >

      <v-card
          :rounded="false"
          title="My Monitors"
      >
        <template #prepend>
          <v-icon color="#cc8f53">mdi-robot</v-icon>
        </template>
        <template #append>
          <v-btn @click="addMonitor"
                 color="primary"
                 text="Add"
                 density="comfortable"
                 class="text-none"
                 prepend-icon="mdi-plus"
          ></v-btn>
        </template>
        <v-card-item style="height: calc(100% - 64px);">
          <div class="full-width full-height overflow-y-auto" style="height: 100%;">
            <v-table hover>
              <thead>
              <tr>
                <th class="key-col text-left font-weight-bold">
                  Key
                </th>
                <th class="prefix-col text-left font-weight-bold">
                  Prefix
                </th>
                <th class="status-col text-left font-weight-bold">
                  Watch Status
                </th>
                <th class="op-col text-left">
                  <v-text-field
                      v-model="searchKeyword"
                      density="compact"
                      label="Search"
                      prepend-inner-icon="mdi-magnify"
                      variant="solo-filled"
                      flat
                      hide-details
                      single-line
                  ></v-text-field>
                </th>
              </tr>
              </thead>
              <tbody>
              <tr v-for="monitor in monitorList"
                  :key="monitor.key"
              >
                <td class="key-col">{{ monitor.key }}</td>
                <td class="prefix-col">
                  <v-chip
                      v-if="monitor.isPrefix"
                      size="small"
                      color="success"
                      variant="outlined"
                  >
                    Yes
                  </v-chip>
                  <v-chip
                      v-else
                      size="small"
                      color="secondary"
                      variant="outlined"
                  >
                    No
                  </v-chip>

                </td>
                <td class="status-col">
                  <v-chip
                      v-if="monitor.paused"
                      size="small"
                      color="secondary"
                      variant="outlined"
                      style="width: 78px;"
                  >
                    <v-icon icon="mdi-robot-dead-outline" start></v-icon>
                    Paused
                  </v-chip>
                  <v-chip
                      v-else
                      size="small"
                      color="success"
                      variant="outlined"
                      style="width: 78px;"
                  >
                    <v-icon icon="mdi-robot-happy" start></v-icon>
                    Running
                  </v-chip>
                </td>
                <td class="op-col">
                  <v-btn text="Edit"
                         color="primary"
                         class="text-none"
                         size="small"
                         prepend-icon="mdi-pencil"
                         @click="editKeyMonitor(monitor.key)"
                  ></v-btn>
                  <v-btn :text="monitor.paused ? 'Start' : 'Stop'"
                         :color="monitor.paused ? 'yellow' : 'green'"
                         class="text-none ml-2"
                         size="small"
                         :prepend-icon="monitor.paused ? 'mdi-play-circle-outline' : 'mdi-stop-circle-outline'"
                         @click="pauseMonitor(monitor, !monitor.paused)"
                  ></v-btn>
                  <v-btn text="Remove"
                         color="red"
                         class="text-none ml-2"
                         size="small"
                         prepend-icon="mdi-delete"
                         @click="removeMonitor(monitor.key)"
                  ></v-btn>
                </td>
              </tr>
              </tbody>
            </v-table>
          </div>
        </v-card-item>
      </v-card>
    </v-dialog>
  </div>
</template>

<style scoped lang="scss">
.event-list-enter-active,
.event-list-leave-active {
  transition: all 0.5s ease;
}

.event-list-enter-from,
.event-list-leave-to {
  opacity: 0;
  transform: translateX(30px);
}

.op-col {
  min-width: 100px;
}

.diff-kv-info {
  margin: 15px 0;
  padding: 5px;
}
.diff-kv-info:hover {
  background-color: rgba(227, 225, 225, 0.3);
}
</style>