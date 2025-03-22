<script setup lang="ts">

import { appWindow } from "@tauri-apps/api/window";
import { computed, onMounted, reactive, ref } from "vue";
import { _useUpdateInfo } from "~/common/store.ts";
import { _checkUpdateAndInstall } from "~/common/updater.ts";
import { _openSettingWindow } from "~/common/windows.ts";
import SnapshotList from "~/components/SnapshotList.vue";

const maximize = ref(false)

const props = defineProps({
  height: Number,
  windowLabel: {
    type: String,
    required: true
  }
})

const title = ref<string>('Etcd Workbench')
const updateInfo = _useUpdateInfo();
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
  maximize.value = await appWindow.isMaximized() || await appWindow.isFullscreen()
})

// const toggleMaximize = async () => {
//   await appWindow.setFullscreen(!(await appWindow.isFullscreen()))
//   maximize.value = !maximize.value
// }
//
// const closeApp = () => {
//   appWindow.close()
// }

const setting = async () => {
  _openSettingWindow()
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
                data-tauri-drag-region
                class="pr-0"
  >
<!--    <span class="system-native-btn-group" >-->
<!--      <span @click="closeApp"-->
<!--            class="system-native-btn bg-red"-->
<!--      >-->
<!--        <svg class="system-native-icon"-->
<!--             viewBox="0 0 1024 1024"-->
<!--             xmlns="http://www.w3.org/2000/svg"-->
<!--             width="8"-->
<!--             height="8">-->
<!--          <path-->
<!--              d="M512 615.424l314.002286 314.002286a73.142857 73.142857 0 0 0 103.424-103.424L615.424 512l314.002286-314.002286a73.142857 73.142857 0 1 0-103.424-103.424L512 408.576 197.997714 94.573714a73.142857 73.142857 0 1 0-103.424 103.424L408.576 512l-314.002286 314.002286a73.142857 73.142857 0 0 0 103.424 103.424L512 615.424z"-->
<!--              fill="#760E0E">-->
<!--          </path>-->
<!--        </svg>-->
<!--      </span>-->
<!--      <span class="system-native-btn bg-yellow ml-2"-->
<!--            @click="appWindow.minimize()"-->
<!--      >-->
<!--        <svg class="system-native-icon"-->
<!--             viewBox="0 0 1024 1024"-->
<!--             xmlns="http://www.w3.org/2000/svg"-->
<!--             width="8"-->
<!--             height="8">-->
<!--          <path v-show="!maximize"-->
<!--                d="M0 439.008h1024v146.016H0v-146.016z"-->
<!--                fill="#760E0E">-->
<!--          </path>-->
<!--        </svg>-->
<!--      </span>-->
<!--      <span @click="toggleMaximize"-->
<!--            class="system-native-btn bg-green ml-2"-->
<!--      >-->
<!--        <svg v-if="maximize"-->
<!--             class="system-native-icon"-->
<!--             viewBox="0 0 1024 1024"-->
<!--             xmlns="http://www.w3.org/2000/svg"-->
<!--             width="8"-->
<!--             height="8">-->
<!--          <path-->
<!--              d="M23.93216 512h440.704c26.24 0 47.616 21.376 47.616 47.616v440.704a23.808 23.808 0 0 1-40.576 16.64L7.16416 552.704A23.808 23.808 0 0 1 23.93216 512z m976.64 0H559.86816A47.616 47.616 0 0 1 512.25216 464.384V23.68a23.808 23.808 0 0 1 40.576-16.64l464.512 464.384A23.808 23.808 0 0 1 1000.57216 512z"-->
<!--              fill="#760E0E"></path>-->
<!--        </svg>-->
<!--        <svg v-else-->
<!--             class="system-native-icon"-->
<!--             viewBox="0 0 1024 1024"-->
<!--             xmlns="http://www.w3.org/2000/svg"-->
<!--             width="8"-->
<!--             height="8">-->
<!--          <path-->
<!--              d="M903.68 120.32l30.72 578.048c1.024 22.016-26.624 37.376-40.448 24.064L284.672 141.312c-14.848-13.824 0-41.984 20.992-42.496l578.048 3.072c11.264-0.512 18.944 6.656 19.968 18.432z m0 0M139.776 920.576l578.048 4.608c22.016 0 35.84-28.16 22.016-41.472L131.072 302.592c-14.848-13.824-41.984 2.048-41.472 23.04l31.232 576.512c0 10.752 7.68 17.92 18.944 18.432z m0 0"-->
<!--              fill="#760E0E">-->
<!--          </path>-->
<!--        </svg>-->
<!--      </span>-->
<!--    </span>-->


    <v-spacer></v-spacer>

    <div v-if="windowLabel == 'main'">
      <v-btn v-if="updateInfo.valid"
             class="text-none ms-2 pr-2"
             color="light-green-darken-1"
             text="New Version"
             variant="outlined"
             rounded
             density="comfortable"
             prepend-icon="mdi-bell-ring-outline"
             size="small"
             @click="_checkUpdateAndInstall"
      ></v-btn>

      <SnapshotList v-show="showSnapshotList"
                    @length-changed="snapshotListLenChanged"
                    @show-changed="snapshotListShowChanged"
      ></SnapshotList>

      <v-btn class="me-2"
             icon="mdi-cog"
             size="small"
             variant="text"
             :rounded="false"
             density="comfortable"
             title="Settings"
             :ripple="false"
             @click="setting"
      ></v-btn>
    </div>

  </v-system-bar>
</template>

<style scoped lang="scss">
@import "~/styles/variables";

$--mac-native-btn-width: 12px;

.system-native-btn {
  width: $--mac-native-btn-width;
  height: $--mac-native-btn-width;
  line-height: $--mac-native-btn-width;
  border-radius: 15px;
  background-color: red;
  font-size: 12px;
  display: inline-block;
  padding: 0;
  text-align: center;
  cursor: pointer;

  .system-native-icon {
    opacity: 0;
    transition: linear .8ms all;
  }
}

.system-native-btn-group {
  display: inline-block;
}

.system-native-btn-group:hover {
  .system-native-icon {
    opacity: 1;
  }
}
</style>
