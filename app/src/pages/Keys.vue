<script setup lang="ts">

import {_getAllKeys} from "~/common/services.ts";
import {_tipError} from "~/common/events.ts";
import {onMounted, PropType, reactive, ref} from "vue";
import {SessionData} from "~/common/transport/connection.ts";
import DragBox from "~/components/DragBox.vue";
import DragItem from "~/components/DragItem.vue";

const props = defineProps({
  session: {
    type: Object as PropType<SessionData>,
    required: true
  }
})

const treeValue = ref([])
const treeData = ref([
  {
    id: 2,
    title: 'node_modules',
  },
  {
    id: 3,
    title: 'public',
    children: [
      {
        id: 4,
        title: 'static',
        children: [{
          id: 5,
          title: 'logo.png',
          file: 'png',
        }],
      },
      {
        id: 6,
        title: 'favicon.ico',
        file: 'png',
      },
      {
        id: 7,
        title: 'index.html',
        file: 'html',
      },
    ],
  },
  {
    id: 12,
    title: 'vue.config.js',
    file: 'js',
  },
  {
    id: 13,
    title: 'yarn.lock',
    file: 'txt',
  },
])
const treeSelectable = ref(false)
const fileIcon = reactive({
  html: 'mdi-language-html5',
  js: 'mdi-nodejs',
  json: 'mdi-code-json',
  md: 'mdi-language-markdown',
  pdf: 'mdi-file-pdf-box',
  png: 'mdi-file-image',
  txt: 'mdi-file-document-outline',
  xls: 'mdi-file-excel',
})

onMounted(() => {
  loadAllKeys()
})

const loadAllKeys = () => {
  _getAllKeys(props.session?.id).then(data => {
    console.log(data)
  }).catch(e => {
    _tipError(e)
  })
}

const addKey = () => {

}

const deleteKey = () => {

}

const treeSelected = (item) => {
  if (treeSelectable.value) {
    console.log("==>",treeValue)
  } else {
    console.log("select tree item", item)
  }
}

const toggleTreeSelectable = () => {
  treeValue.value = []
  treeSelectable.value = !treeSelectable.value
}

</script>

<template>
  <div class="fill-height overflow-y-auto">
    <div class="action-area">
      <v-btn class="text-none"
             prepend-icon="mdi-refresh"
             color="primary"
             @click="loadAllKeys"
      >Refresh</v-btn>
      <v-btn class="text-none ml-2"
             prepend-icon="mdi-file-document-plus-outline"
             color="green"
             @click="addKey"
      >
        Add Key
      </v-btn>
      <v-btn class="text-none ml-2"
             :prepend-icon="treeSelectable ? 'mdi-checkbox-outline' : 'mdi-checkbox-blank-outline'"
             color="secondary"
             @click="toggleTreeSelectable"
      >
        Select Key
      </v-btn>
      <v-btn class="text-none ml-2"
             v-show="treeSelectable"
             prepend-icon="mdi-file-document-minus-outline"
             color="red"
             @click="deleteKey"
      >
        Delete Key
      </v-btn>
    </div>
    <v-layout class="main-area">
      <drag-box>
        <drag-item class="overflow-y-auto" style="width: 300px">
          <v-treeview
              :items="treeData"
              :open-strategy="treeSelectable ? 'multiple' : 'single'"
              item-key="id"
              :selectable="treeSelectable"
              :selected="treeValue"
              :select-strategy="treeSelectable ? 'leaf' : 'single-leaf'"
              return-object
              slim
              density="compact"
              @click:select="treeSelected"
              class="user-select-none"
          >
            <template v-slot:prepend="{ item, open }">
              <v-icon v-if="!item.file">
                {{ open ? 'mdi-folder-open' : 'mdi-folder' }}
              </v-icon>
              <v-icon v-else>
                {{ fileIcon[item.file] }}
              </v-icon>
            </template>
          </v-treeview>
        </drag-item>
        <drag-item style="width: calc(100% - 300px)" :show-resize-line="false">

          <div style="width: 100%; height: 50px; background: red;">

          </div>
        </drag-item>
      </drag-box>
    </v-layout>
  </div>
</template>

<style scoped lang="scss">
$--action-area-height: 50px;
$--action-area-margin-bottom: 10px;

.action-area {
  height: $--action-area-height;
  padding: 10px;
  margin-bottom: $--action-area-margin-bottom;
}
.main-area {
  height: calc(100% - $--action-area-height - $--action-area-margin-bottom);
}
</style>