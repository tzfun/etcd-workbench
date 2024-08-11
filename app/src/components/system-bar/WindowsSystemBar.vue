<script setup lang="ts">

import etcdLogo from "~/assets/etcd.png";
import {_goBrowserPage} from "~/common/utils.ts";
import {appWindow} from "@tauri-apps/api/window";
import {_confirm} from "~/common/events.ts";
import {onMounted, ref} from "vue";
import {useTheme} from "vuetify";

const maximize = ref(false)
const theme = useTheme()

defineProps({
  title: String,
  height: Number
})

const emits = defineEmits(['setting', 'show-info'])

onMounted(async () => {
  maximize.value = await appWindow.isMaximized()
})

const closeApp = () => {
  _confirm("Exist Workbench", "Are you sure you want to close the app?").then(() => {
    appWindow.close()
  }).catch(() => {
  })
}

const toggleMaximize = async () => {
  await appWindow.toggleMaximize()
  maximize.value = !maximize.value
}

const showAppInfo = () => {
  emits('show-info')
}

const setting = () => {
  emits('show-info')
}

const toggleTheme = () => {
  theme.global.name.value = theme.global.current.value.dark ? 'light' : 'dark'
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

    <v-btn class="system-extend-btn ms-2"
           icon="mdi-cog"
           size="small"
           variant="text"
           :rounded="false"
           density="comfortable"
           title="Settings"
           :ripple="false"
           @click="setting"
    ></v-btn>
    <v-btn class="system-extend-btn ms-2"
           icon="mdi-github"
           size="small"
           variant="text"
           :rounded="false"
           density="comfortable"
           title="Fork on GitHub"
           :ripple="false"
           @click="_goBrowserPage('https://github.com/tzfun/etcd-workbench')"
    ></v-btn>
    <v-btn class="system-extend-btn ms-2"
           icon="mdi-information-variant-circle"
           size="small"
           variant="text"
           :rounded="false"
           density="comfortable"
           title="About"
           :ripple="false"
           @click="showAppInfo"
    ></v-btn>
    <v-btn class="system-extend-btn ms-2"
           icon="mdi-brightness-6"
           size="small"
           variant="text"
           :rounded="false"
           density="comfortable"
           title="About"
           :ripple="false"
           @click="toggleTheme"
    ></v-btn>

    <v-spacer></v-spacer>

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
.system-extend-btn {
  font-size: 1.1em;
}

.system-native-btn {
  font-size: 0.9em;
  opacity: 0.5;
}

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
  color: #757575
}

</style>
