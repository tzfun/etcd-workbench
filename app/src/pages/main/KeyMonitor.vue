<script setup lang="ts">
import {computed, onMounted, PropType, reactive, ref} from "vue";
import {SessionData} from "~/common/transport/connection.ts";
import {_confirmSystem, _emitLocal, _listenLocal, EventName, KeyMonitorEvent} from "~/common/events.ts";
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
    type: Array<KeyMonitorEvent>,
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

const read = (e: KeyMonitorEvent) => {
  if (!e.read) {
    e.read = true
    emits('on-read', 1)
  }

  if (e.eventType == 'ValueChange') {
    valueDiffDialog.key = e.key
    valueDiffDialog.beforeValue = _decodeBytesToString(e.previous)
    valueDiffDialog.afterValue = _decodeBytesToString(e.current)

    let validContent = _isEmpty(valueDiffDialog.beforeValue) ? valueDiffDialog.afterValue : valueDiffDialog.beforeValue
    let editorLang = _tryParseEditorLanguage(e.key, validContent, props.session?.namespace)
    valueDiffDialog.language = _tryParseDiffLanguage(editorLang)
    valueDiffDialog.show = true
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

const addMonitor = () => {
  _emitLocal(EventName.EDIT_KEY_MONITOR, {
    session: props.session?.id,
    edit: false
  })
}

</script>

<template>
  <div class="fill-height pa-5 overflow-y-auto">
    <div>
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
    </div>
    <div style="height: calc(100% - 56px); overflow-y: auto;" class="mt-5">
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
              <v-icon v-else-if="e.eventType == 'LeaseChange'">mdi-clock-time-nine</v-icon>
              <v-icon v-else-if="e.eventType == 'ValueChange'">mdi-content-save-all-outline</v-icon>
            </template>

            <template #append>
              <span v-if="e.eventType == 'Create'" class="text-medium-emphasis">Created</span>
              <span v-else-if="e.eventType == 'Remove'" class="text-medium-emphasis">Removed</span>
              <span v-else-if="e.eventType == 'LeaseChange'" class="text-medium-emphasis">Lease Changed</span>
              <span v-else-if="e.eventType == 'ValueChange'" class="text-medium-emphasis">Value Changed</span>
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
          <div style="height: 100%;overflow: auto;">
            <Tree ref="kvMonitorTree"
                  :tree-id="`kv-collection-tree-${new Date().getTime()}`"
                  :key-splitter="KEY_SPLITTER"
                  :session="session"
                  :show-node-suffix="false"
                  :show-check-box="false"
                  show-hover-remove
                  :enable-search="false"
                  :enable-select="false"
                  style="width: max-content;"
                  :init-items="Object.keys(session.keyMonitorMap!)"
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