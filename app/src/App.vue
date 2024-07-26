<script setup lang="ts">
import {appWindow} from '@tauri-apps/api/window'
import etcdLogo from '~/assets/etcd.png';
import {_confirm, events} from "~/common/events.ts";
import {DialogItem, TipsItem} from "~/common/types.ts";
import {onMounted, ref} from "vue";
import {Platform, platform as getPlatform} from "@tauri-apps/api/os";

const loading = ref(false)
const dialogs = ref<DialogItem[]>([])
const tips = ref<TipsItem[]>([])
const platform = ref<Platform>('win32')

onMounted(async () => {

  platform.value = await getPlatform()

  //  频闭右键事件
  disableRightMenu()

  events.on('loading', (state) => {
    loading.value = !!state;
  })

  events.on('dialog', (param) => {
    let dialog = param as DialogItem
    let idx = -1;
    for (let i = 0; i < dialogs.value.length; i++) {
      if (!dialogs.value[i].value) {
        idx = i;
        break
      }
    }

    dialog.value = true
    if (idx < 0) {
      dialogs.value.push(dialog)
    } else {
      dialogs.value[idx] = dialog
    }
  })

  events.on('tip', (param) => {
    let tip = param as TipsItem
    let idx = -1;
    for (let i = 0; i < tips.value.length; i++) {
      if (!tips.value[i].value) {
        idx = i;
        break
      }
    }

    tip.value = true
    if (idx < 0) {
      tips.value.push(tip)
    } else {
      tips.value[idx] = tip
    }
  })
})

const disableRightMenu = () => {

  if (window.location.hostname !== "tauri.localhost") {
    return
  }

  document.addEventListener('keydown', e => {
    if (e.ctrlKey && e.key.toLowerCase() == 'r') {
      e.preventDefault()
      return false
    }
  }, {capture: true})

  document.addEventListener('contextmenu', e => {
    e.preventDefault()
    return false
  }, {capture: true})
}

const closeApp = () => {
  _confirm("Are you sure you want to close the app?").then(() => {
    appWindow.close()
  }).catch(() => {
  })
}

</script>

<template>
  <v-app id="vuetify-app">
    <v-layout style="height: 50px">
      <v-system-bar window
                    v-if="platform == 'win32'"
                    :height="28"
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
        <span class="user-select-none">ETCD Workbench</span>

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
               size="small"
               icon="mdi-checkbox-blank-outline"
               variant="text"
               :rounded="false"
               density="comfortable"
               @click="appWindow.toggleMaximize()"
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
      <v-main class="fill-height">

      </v-main>
    </v-layout>

    <!--    全局公共组件    -->

    <v-overlay
        :model-value="loading"
        class="align-center justify-center"
    >
      <v-progress-circular
          color="primary"
          size="64"
          indeterminate
      ></v-progress-circular>
    </v-overlay>

    <v-dialog
        v-for="(item, key) in dialogs"
        :key="key"
        v-model="item.value"
        persistent
        width="auto"
    >
      <v-card
          max-width="500"
          min-width="400"
          :prepend-icon="item.icon"
          :text="item.content"
          :title="item.title"
      >
        <template v-slot:actions v-if="item.buttons">
          <v-btn
              v-for="(btn,k ) in item.buttons"
              :key="k"
              :class="btn.class"
              :text="btn.text"
              @click="btn.callback(item, $event)"
          ></v-btn>
        </template>
      </v-card>
    </v-dialog>

    <v-snackbar
        v-for="(item, key) in tips"
        :key="key"
        v-model="item.value"
        location="top"
        :content-class="item.class"
    >
      <v-icon v-if="item.icon">{{ item.icon }}</v-icon>
      {{ item.content }}

      <template v-slot:actions v-if="item.close">
        <v-btn
            color="pink"
            variant="text"
            icon="mdi-close"
            @click="item.value = false; item.close()"
        />
      </template>
    </v-snackbar>
  </v-app>
</template>

<style scoped>
.system-native-btn {
  font-size: 1em;
  color: black;
}

.system-native-btn-close:hover {
  background-color: #D50000;
  color: white;
}
</style>
