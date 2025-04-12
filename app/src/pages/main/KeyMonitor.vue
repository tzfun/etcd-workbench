<script setup lang="ts">
import {computed, onMounted, PropType, reactive, ref} from "vue";
import {SessionData, KeyMonitorConfig} from "~/common/transport/connection.ts";
import {_confirmSystem, _emitLocal, _listenLocal, EventName, KeyWatchEvent} from "~/common/events.ts";
import {
  _decodeBytesToString,
  _isEmpty,
  _timeFormat,
  _tryParseDiffLanguage,
  _tryParseEditorLanguage
} from "~/common/utils.ts";
import {CodeDiff} from "v-code-diff";
import {useTheme} from "vuetify";
import Tree from "~/components/tree/Tree.vue";
import {_useSettings} from "~/common/store.ts";
import {_handleError, _removeKeyMonitor} from "~/common/services.ts";
import { EditorHighlightLanguage } from "~/common/types";

const theme = useTheme()

const valueDiffDialog = reactive({
  show: false,
  key: <string>"",
  beforeValue: <string>"",
  afterValue: <string>"",
  language: <string>""
})
const monitorTreeDialog = ref(false)
const kvMonitorTree = ref<InstanceType<typeof Tree>>()
const isDarkTheme = computed<boolean>(() => {
  return theme.global.name.value === 'dark'
})
const KEY_SPLITTER = computed<string>(() => {
  return _useSettings().value.kvPathSplitter
})

const emits = defineEmits(['on-read'])
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

onMounted(() => {
  _listenLocal(EventName.KEY_MONITOR_CONFIG_CHANGE, e => {
    if (e.session == props.session?.id) {
      let key = e.key as string
      if (e.type == 'create') {
        kvMonitorTree.value?.addItemToTree(key)
      } else if (e.type == 'remove') {
        kvMonitorTree.value?.removeItemFromTree(key)
      }
    }

  })
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
    valueDiffDialog.key = e.key

    let editorLang;
    if(e.prevKv && e.curKv) {
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
             @click="monitorTreeDialog = true"
             color="#cc8f53"
      >My Monitors
      </v-btn>

      <v-spacer/>

<!--      <v-btn class="text-none my-2"-->
<!--             density="comfortable"-->
<!--             variant="tonal"-->
<!--             size="small"-->
<!--             :icon="session.keyMonitorPaused ? 'mdi-play' : 'mdi-pause'"-->
<!--             :title="session.keyMonitorPaused ? 'Start the monitor' : 'Stop the monitor'"-->
<!--             @click="togglePauseState"-->
<!--             :loading="togglePauseLoading"-->
<!--      ></v-btn>-->
<!--      <v-chip-->
<!--        v-if="session.keyMonitorPaused"-->
<!--        class="ma-2"-->
<!--        size="small"-->
<!--        color="secondary"-->
<!--        variant="outlined"-->
<!--        style="width: 78px;"-->
<!--      >-->
<!--        <v-icon icon="mdi-robot-dead-outline" start></v-icon>-->
<!--        Paused-->
<!--      </v-chip>-->
<!--      <v-chip-->
<!--        v-else-->
<!--        class="ma-2"-->
<!--        size="small"-->
<!--        color="success"-->
<!--        variant="outlined"-->
<!--        style="width: 78px;"-->
<!--      >-->
<!--        <v-icon icon="mdi-robot-happy" start></v-icon>-->
<!--        Running-->
<!--      </v-chip>-->
      
    </v-layout>
    <div style="height: calc(100% - 56px); overflow-y: auto;">
      <v-list class="pa-0 overflow-hidden"
              :selectable="false"
              lines="two"
              v-if="events.length > 0"
      >
        <transition-group name="event-list">
          <v-list-item
              v-for="e in events"
              :key="e.id"
              :title="e.key"
              :subtitle="_timeFormat(e.eventTime)"
              @click="read(e)"
              density="comfortable"
              :variant="e.read ? 'plain' : 'tonal'"
          >
            <template #prepend>
              <v-icon v-if="e.eventType == 'Create'">mdi-folder-plus-outline</v-icon>
              <v-icon v-else-if="e.eventType == 'Remove'">mdi-folder-remove-outline</v-icon>
              <v-icon v-else-if="e.eventType == 'Modify'">mdi-content-save-all-outline</v-icon>
            </template>

            <template #append>
              <span v-if="e.eventType == 'Create'" class="text-medium-emphasis">Created</span>
              <span v-else-if="e.eventType == 'Remove'" class="text-medium-emphasis">Removed</span>
              <span v-else-if="e.eventType == 'Modify'" class="text-medium-emphasis">Value Changed</span>
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
        v-model="monitorTreeDialog"
        eager
        transition="slide-x-reverse-transition"
        scrollable
        class="collection-drawer"
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
            <Tree ref="kvMonitorTree"
                  :tree-id="`kv-collection-tree-${new Date().getTime()}`"
                  :key-splitter="KEY_SPLITTER"
                  :session="session"
                  :show-node-suffix="false"
                  :show-check-box="false"
                  show-hover-remove
                  style="height: 100%;"
                  :enable-select="false"
                  :init-items="Object.keys(session.keyMonitorMap!)"
                  @on-click="editKeyMonitor"
                  @on-click-remove="removeMonitor"
            ></Tree>
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
</style>