<script setup lang="ts">
import {computed, PropType, reactive} from "vue";
import {SessionData} from "~/common/transport/connection.ts";
import {_confirmSystem, KeyMonitorEvent} from "~/common/events.ts";
import {
  _decodeBytesToString,
  _isEmpty,
  _timeFormat,
  _tryParseDiffLanguage,
  _tryParseEditorLanguage
} from "~/common/utils.ts";
import {CodeDiff} from "v-code-diff";
import {useTheme} from "vuetify";

const theme = useTheme()

const valueDiffDialog = reactive({
  show: false,
  key: <string>"",
  beforeValue: <string>"",
  afterValue: <string>"",
  language: <string>""
})
const isDarkTheme = computed<boolean>(() => {
  return theme.global.name.value === 'dark'
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
      props.events?.slice(0, len)
    }
    markAllRead()
  }).catch(() => {
  })
}
</script>

<template>
  <div class="fill-height pa-5 overflow-y-auto">
    <div>
      <v-btn class="text-none"
             prepend-icon="mdi-checkbox-marked-circle-auto-outline"
             @click="markAllRead"
             color="primary"
      >Mark All Read
      </v-btn>
      <v-btn class="text-none ml-2"
             prepend-icon="mdi-delete-circle-outline"
             @click="clearHistory"
             color="red"
      >Clear History
      </v-btn>
    </div>
    <div>
      <v-list class="my-5 pa-0"
              :selectable="false"
              lines="two"
              v-if="events.length > 0"
      >
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
      </v-list>

      <v-empty-state v-else
                     icon="mdi-package-variant"
                     headline="No Monitor Event"
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
  </div>
</template>

<style scoped lang="scss">

</style>