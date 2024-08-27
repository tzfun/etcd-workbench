<script setup lang="ts">

import etcdLogo from "~/assets/etcd.png";
import {appWindow} from "@tauri-apps/api/window";
import {computed, onMounted, reactive, ref} from "vue";
import {_openSettingWindow} from "~/common/windows.ts";
import SnapshotList from "~/components/SnapshotList.vue";
import {_checkUpdateAndInstall, _emitGlobal, EventName} from "~/common/events.ts";
import {_useUpdateInfo} from "~/common/store.ts";

const maximize = ref(false)

const props = defineProps({
  height: Number,
  windowLabel: {
    type: String,
    required: true
  }
})

const updateInfo = _useUpdateInfo();

const title = ref<string>('Etcd Workbench')

const snapshotListState = reactive({
  show: false,
  len: 0
})

const showSnapshotList = computed<boolean>(() => {
  return snapshotListState.show || snapshotListState.len > 0
})

onMounted(async () => {
  switch (props.windowLabel) {
    case 'main':
      title.value = 'Etcd Workbench'
      break
    case 'setting':
      title.value = 'Settings'
      break
  }

  maximize.value = await appWindow.isMaximized()
})



const closeApp = () => {
  if (props.windowLabel === 'main') {
    _emitGlobal(EventName.CONFIRM_EXIT, null);
  } else {
    appWindow.hide()
  }
}

const toggleMaximize = async () => {
  await appWindow.toggleMaximize()
  maximize.value = !maximize.value
}

const snapshotListLenChanged = (len: number) => {
  snapshotListState.len = len
}

const snapshotListShowChanged = (show: boolean) => {
  snapshotListState.show = show
}

</script>

<template>
  <v-system-bar window
                :height="height"
                @dblclick.self="appWindow.toggleMaximize()"
                data-tauri-drag-region
                class="pr-0"
  >
    <v-icon class="me-2">
      <v-img :src="etcdLogo"
             cover
             :width="30"
             :height="30"
      ></v-img>
    </v-icon>
    <span class="user-select-none">{{ title }}</span>

    <v-spacer></v-spacer>

    <div v-if="windowLabel == 'main'">

      <v-btn v-if="updateInfo.valid"
             class="system-extend-btn text-none ms-2 pl-2 pr-2"
             color="light-green-darken-1"
             text="Update Workbench"
             variant="outlined"
             rounded
             prepend-icon="mdi-bell-ring-outline"
             density="comfortable"
             size="small"
             @click="_checkUpdateAndInstall"
      ></v-btn>

      <SnapshotList v-show="showSnapshotList"
                    @length-changed="snapshotListLenChanged"
                    @show-changed="snapshotListShowChanged"
      ></SnapshotList>

      <v-btn class="system-extend-btn ms-2"
             icon="mdi-cog"
             size="small"
             variant="text"
             :rounded="false"
             density="comfortable"
             title="Settings"
             :ripple="false"
             @click="_openSettingWindow"
      ></v-btn>
    </div>
    <v-divider vertical
               class="mr-2 ml-2"
               length="80%"
               style="margin-top: 3px;"
               v-if="windowLabel == 'main'"
    ></v-divider>

    <v-btn class="system-native-btn"
           icon="mdi-minus"
           size="small"
           variant="text"
           :rounded="false"
           density="comfortable"
           @click="appWindow.minimize()"
    ></v-btn>
    <v-btn class="system-native-btn ms-2"
           style="transform: rotate(90deg);"
           size="small"
           :icon="maximize ? 'mdi-vector-arrange-below' : 'mdi-checkbox-blank-outline'"
           variant="text"
           :rounded="false"
           density="comfortable"
           @click="toggleMaximize"
    ></v-btn>
    <v-btn class="system-native-btn system-native-btn-close ms-2"
           size="small"
           icon="mdi-close"
           variant="text"
           :rounded="false"
           density="comfortable"
           @click="closeApp"
    ></v-btn>
  </v-system-bar>
</template>

<style scoped lang="scss">
@import "~/styles/variables";

.system-extend-btn {
  font-size: 1.1em;
}

.system-native-btn {
  font-size: 0.9em;
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
