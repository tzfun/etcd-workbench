<script setup lang="ts">

import {appWindow} from "@tauri-apps/api/window";
import {computed, onMounted, PropType, reactive, ref} from "vue";
import {_confirm, _emitGlobal, EventName} from "~/common/events.ts";
import {_openSettingWindow, isMaximizeState} from "~/common/windows.ts";
import SnapshotList from "~/components/SnapshotList.vue";
import {UpdateInfo} from "~/common/types.ts";
import {_byteTextFormat} from "~/common/utils.ts";
import {relaunch} from "@tauri-apps/api/process";
import {_checkUpdate} from "~/common/updater.ts";
import {useI18n} from "vue-i18n";

const props = defineProps({
  height: Number,
  windowLabel: {
    type: String,
    required: true
  },
  updateInfo: {
    type: Object as PropType<UpdateInfo>,
    required: true
  }
})

const {t} = useI18n();
//  只有主窗口才允许全屏
const enableMaximize: boolean = props.windowLabel == 'main';

const title = ref<string>('Etcd Workbench')

const snapshotListState = reactive({
  show: false,
  len: 0
})

const showSnapshotList = computed<boolean>(() => {
  return snapshotListState.show || snapshotListState.len > 0
})
const downloadingProgress = computed(() => {
  if (props.updateInfo?.state == 'downloading') {
    if (props.updateInfo.contentLength && props.updateInfo.contentLength > 0) {
      return 100 * props.updateInfo.chunkLength / props.updateInfo.contentLength
    }
  }
  return 0
})

onMounted(async () => {
  switch (props.windowLabel) {
    case 'main':
      title.value = t('window.main')
      break
    case 'setting':
      title.value = t('window.settings')
      break
  }
})

const closeApp = () => {
  if (props.windowLabel === 'main') {
    _emitGlobal(EventName.CONFIRM_EXIT, null);
  } else {
    appWindow.hide()
  }
}

const toggleMaximize = async () => {
  if (enableMaximize) {
    await appWindow.toggleMaximize()
  }
}

const snapshotListLenChanged = (len: number) => {
  snapshotListState.len = len
}

const snapshotListShowChanged = (show: boolean) => {
  snapshotListState.show = show
}

const confirmRestart = () => {
  _confirm(t('common.restart'),t('feedback.updateRestartConfirm')).then(() => {
    relaunch()
  }).catch(() => {})
}
</script>

<template>
  <v-system-bar
      window
      :height="height"
      @dblclick.self="toggleMaximize()"
      data-tauri-drag-region
      class="pr-0"
  >
    <v-icon class="me-2">
      <v-img src="/logo.png" cover :width="30" :height="30"/>
    </v-icon>
    <span class="user-select-none">{{ title }}</span>

    <v-spacer/>

    <div v-if="windowLabel == 'main'">
      <v-chip
          class="mx-2"
          v-if="updateInfo.state == 'available'"
          variant="outlined"
          size="small"
          density="comfortable"
          color="light-green-darken-1"
          prepend-icon="mdi-bell-ring-outline"
          @click="_checkUpdate"
      >
        New Version
      </v-chip>
      <span class="mx-2"
            v-else-if="updateInfo.state == 'downloading'"
      >
        <v-progress-circular
            v-model="downloadingProgress"
            size="20"
            color="blue-lighten-3"
            class="mr-2"
            width="2"
        >
          <v-icon size="small">mdi-arrow-down-bold</v-icon>
        </v-progress-circular>
        <strong class="mr-2">{{ Math.ceil(downloadingProgress) }}%</strong>
        {{ _byteTextFormat(updateInfo.chunkLength) }} / {{ _byteTextFormat(updateInfo.contentLength) }}
      </span>
      <v-chip
          v-else-if="updateInfo.state == 'downloaded'"
          class="mx-2"
          variant="outlined"
          size="small"
          density="comfortable"
          color="blue-lighten-3"
          prepend-icon="mdi-download"
          :text="t('common.downloaded')"
      />
      <v-chip
          v-else-if="updateInfo.state == 'installed'"
          class="mx-2"
          variant="outlined"
          size="small"
          density="comfortable"
          color="light-green-darken-1"
          prepend-icon="mdi-check-bold"
          @click="confirmRestart"
          :text="t('common.installedUpdates')"
      />

      <SnapshotList v-show="showSnapshotList"
                    @length-changed="snapshotListLenChanged"
                    @show-changed="snapshotListShowChanged"
      />

      <v-btn class="system-extend-btn ms-2"
             icon="mdi-cog"
             size="small"
             variant="text"
             :rounded="false"
             density="comfortable"
             :title="t('window.settings')"
             :ripple="false"
             @click="_openSettingWindow()"
      />
    </div>
    <v-divider vertical
               class="mr-2 ml-2"
               length="80%"
               style="margin-top: 3px;"
               v-if="windowLabel == 'main'"
    />

    <v-btn
        class="system-native-btn"
        icon="mdi-minus"
        size="small"
        variant="text"
        :rounded="false"
        density="comfortable"
        @click="appWindow.minimize()"
    />
    <v-btn
        class="system-native-btn ms-2"
        style="transform: rotate(90deg);"
        size="small"
        :icon="isMaximizeState ? 'mdi-vector-arrange-below' : 'mdi-checkbox-blank-outline'"
        variant="text"
        :rounded="false"
        density="comfortable"
        :disabled="!enableMaximize"
        @click="toggleMaximize"
    />
    <v-btn
        class="system-native-btn system-native-btn-close ms-2"
        size="small"
        icon="mdi-close"
        variant="text"
        :rounded="false"
        density="comfortable"
        @click="closeApp"
    />
  </v-system-bar>
</template>

<style scoped lang="scss">
@import "~/styles/variables";

.system-extend-btn {
  font-size: 1.1em;
  opacity: 0.8;
}

.system-native-btn {
  font-size: 0.9em;
  opacity: 0.5;
}

.system-extend-btn:hover,
.system-native-btn:hover {
  opacity: 1;
}

.system-native-btn-close:hover {
  background-color: #D50000;
  color: white;
}

.tab-icon-close {
  color: #BDBDBD
}

.tab-icon-close:hover {
  color: white;
}
</style>
