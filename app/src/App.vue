<script setup lang="ts">
import {appWindow} from '@tauri-apps/api/window'
import etcdLogo from '~/assets/etcd.png';
import {_confirm, events} from "~/common/events.ts";
import {DialogItem, TipsItem} from "~/common/types.ts";
import {onMounted, ref} from "vue";
import {platform as getPlatform} from "@tauri-apps/api/os";
import {_goBrowserPage} from "~/common/utils.ts";
import {useTheme} from "vuetify";

const loading = ref(false)
const dialogs = ref<DialogItem[]>([])
const tips = ref<TipsItem[]>([])
const platform = ref<string>('win32')
const activeTab = ref<string>('home')
const tabList = ref([
  {
    name: '连接1',
    session: {
      id: 1,
      user: "xxx",
      root: false
    },
    route: '/connection/1/cluster'
  }
])
const maximize = ref(false)

const theme = useTheme()

onMounted(async () => {

  maximize.value = await appWindow.isMaximized()

  let systemTheme = await appWindow.theme()
  if(systemTheme) {
    theme.global.name.value = systemTheme
  }

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
  _confirm("Exist Workbench","Are you sure you want to close the app?").then(() => {
    appWindow.close()
  }).catch(() => {
  })
}

const toggleMaximize = () => {
  appWindow.toggleMaximize().then(() => {
    maximize.value = !maximize.value
  })
}

const showAppInfo = () => {

}

const setting = () => {

}

const toggleTheme = () => {
  theme.global.name.value = theme.global.current.value.dark ? 'light' : 'dark'
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

      <v-main class="fill-height">

        <v-tabs v-model="activeTab"
                show-arrows
                :height="30"
                density="compact"
                color="primary"
        >
          <v-tab icon="mdi-home"
                 value="home"
                 density="compact"
                 class="text-grey-lighten-1"
                 :ripple="false"
                 :min-width="50"
                 to="/"
          >
            <v-icon>mdi-home</v-icon>
          </v-tab>
          <v-tab v-for="tab in tabList"
                 :key="tab.name"
                 :value="tab.name"
                 class="text-grey-lighten-1"
                 :ripple="false"
                 :to="tab.route"
          >
            {{tab.name}}
            <template v-slot:append>
              <v-icon class="tab-icon-close">mdi-close</v-icon>
            </template>
          </v-tab>
        </v-tabs>
        <v-divider></v-divider>
        <div style="height: calc(100% - 30px);">
          <router-view v-slot="{ Component, route }">
            <keep-alive>
              <component :is="Component" :key="route.path"/>
            </keep-alive>
          </router-view>
        </div>
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
        :persistent="item.persistent == undefined ? true : item.persistent"
        :scrollable="item.scrollable == undefined ? true : item.scrollable"
        width="auto"
    >
      <v-card
          :max-width="item.maxWidth ? item.maxWidth : 500"
          :min-width="item.minWidth ? item.minWidth : 400"
          :text="item.content"
          :title="item.title"
      >
        <template v-slot:prepend>
          <v-icon :color="item.iconColor">{{item.icon}}</v-icon>
        </template>
        <template v-slot:append v-if="item.closeBtn">
          <v-icon class="cursor-pointer" @click="item.value = false">mdi-close</v-icon>
        </template>
        <template v-slot:actions v-if="item.buttons">
          <v-btn
              v-for="(btn,k ) in item.buttons"
              :key="k"
              :class="btn.class + ' text-none'"
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
