<script setup lang="ts">

import etcdLogo from "~/assets/etcd.png";
import {appWindow} from "@tauri-apps/api/window";
import {_confirm} from "~/common/localEvents.ts";
import {onMounted, ref} from "vue";
import {_closeAllWindow, _openSettingWindow} from "~/common/windows.ts";

const maximize = ref(false)

const props = defineProps({
  height: Number,
  windowLabel: {
    type: String,
    required: true
  }
})

const title = ref<string>('Etcd Workbench')

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
    _confirm("Exist Workbench", "Are you sure you want to close the app?").then(() => {
      _closeAllWindow()
    }).catch(() => {
    })
  } else {
    appWindow.hide()
  }
}

const toggleMaximize = async () => {
  await appWindow.toggleMaximize()
  maximize.value = !maximize.value
}

const setting = async () => {
  _openSettingWindow()
}

</script>

<template>
  <v-system-bar window
                :height="height"
                @dblclick="appWindow.toggleMaximize()"
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

    <v-btn class="system-extend-btn ms-2"
           icon="mdi-cog"
           size="small"
           variant="text"
           :rounded="false"
           density="comfortable"
           title="Settings"
           :ripple="false"
           @click="setting"
           v-if="windowLabel == 'main'"
    ></v-btn>

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
