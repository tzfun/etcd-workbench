<script setup lang="ts">

import {_getAllKeys, _getKV, _getKVByVersion, _getKVHistoryVersions, _putKV} from "~/common/services.ts";
import {_tipError, _tipWarn} from "~/common/events.ts";
import {computed, onMounted, PropType, reactive, ref} from "vue";
import {SessionData} from "~/common/transport/connection.ts";
import DragBox from "~/components/DragBox.vue";
import DragItem from "~/components/DragItem.vue";
import {KeyValue} from "~/common/transport/kv.ts";
import Editor from "~/components/editor/Editor.vue";
import {_decodeBytesToString, _encodeStringToBytes} from "~/common/utils.ts";
import {EditorConfig} from "~/common/types.ts";
import {CodeDiff} from "v-code-diff";
import {useTheme} from "vuetify";

const theme = useTheme()

type TreeNode = {
  title: string,
  file: boolean,
  iconKey: string,
  children?: TreeNode[],
  data?: KeyValue
}

type DiffInfo = {
  version: number,
  content: string
}

const KEY_SPLITTER = '/'

const props = defineProps({
  session: {
    type: Object as PropType<SessionData>,
    required: true
  }
})

const treeValue = ref([])
const treeData = ref<TreeNode[]>([])
const treeSelectable = ref(false)
const fileIcon = reactive<Record<string, string>>({
  file: 'mdi-file-document-outline',
  js: 'mdi-nodejs',
  ts: 'mdi-language-typescript',
  json: 'mdi-code-json',
  md: 'mdi-language-markdown',
  sql: 'mdi-database-search',
  xml: 'mdi-file-xml-box',
  yaml: 'mdi-code-block-braces',
  properties: 'mdi-cog'
})
const currentKv = ref<KeyValue>()
const currentKvChanged = ref<boolean>(false)

const editorRef = ref<InstanceType<typeof Editor>>()
const editorConfig = reactive<EditorConfig>({
  disabled: false,
  indentWithTab: true,
  tabSize: 2,
  autofocus: false,
  height: "100%",
  fontSize: "1rem",
  language: 'text'
})

const loadingStore = reactive({
  save: false,
  diff: false
})

const versionDiffInfo = reactive({
  show: false,
  key: '',
  version: 0,
  createRevision: 0,
  modRevision: 0,
  language: 'plaintext',
  versionHistory: <number[]>[],
  A: <DiffInfo>{
    version: 0,
    content: ''
  },
  B: <DiffInfo>{
    version: 0,
    content: ''
  }
})
const isDarkTheme = computed<boolean>(() => {
  return theme.global.name.value === 'dark'
})

onMounted(() => {
  loadAllKeys()
})

const loadAllKeys = () => {
  _getAllKeys(props.session?.id).then(data => {
    treeData.value = constructTreeData(data)
  }).catch(e => {
    _tipError(e)
  })
}

const constructTreeData = (data: KeyValue[]): TreeNode[] => {
  data.sort((o1, o2) => {
    if (o1.key > o2.key) {
      return 1
    } else if (o1.key < o2.key) {
      return -1
    } else {
      return 0
    }
  })

  let root: TreeNode = {
    title: 'root',
    file: false,
    iconKey: 'dir',
    children: []
  }

  for (let kv of data) {
    addKvToTree(kv, root)
  }
  return root.children!
}

const addKvToTree = (kv: KeyValue, root: TreeNode) => {
  let splits = kv.key.split(KEY_SPLITTER)
  let node: TreeNode = root

  for (let i = 1; i < splits.length - 1; i++) {
    const floorName = splits[i]
    let floorNode: TreeNode | undefined = undefined
    if (!node.children) {
      node.children = []
    }
    for (let child of node.children) {
      if (!child.file && child.title === floorName) {
        floorNode = child
      }
    }
    if (!floorNode) {
      floorNode = {
        title: floorName,
        file: false,
        iconKey: 'dir',
        children: []
      }
      node.children.push(floorNode)
    }
    node = floorNode
  }

  let fileName = splits[splits.length - 1]
  let fileNode: TreeNode = {
    title: fileName,
    file: true,
    iconKey: tryParseFileNameToType(fileName, 'file')!,
    data: kv
  }

  node.children?.push(fileNode)
}

const tryParseFileNameToType = (fileName: string, defaultType?: string): string | undefined => {
  let dotIdx = fileName.lastIndexOf(".")
  if (dotIdx >= 0) {
    let type = fileName.substring(dotIdx + 1).toLowerCase()
    switch (type) {
      case 'json':
        return 'json'
      case 'sql':
        return 'sql'
      case 'xml':
      case 'html':
      case 'htm':
        return 'xml'
      case 'yml':
      case 'yaml':
        return 'yaml'
      case 'ts':
      case 'typescript':
        return 'ts'
      case 'js':
      case 'javascript':
        return 'js'
      case 'md':
      case 'markdown':
        return 'md'
      case 'ini':
      case 'conf':
      case 'properties':
        return 'properties'
      default:
        return defaultType
    }
  }

  return defaultType
}

const tryFileContentToType = (content: string): string => {
  let lang = 'text'
  content = content.trimStart()
  if (content.startsWith('<')) {
    lang = 'xml'
  } else if (content.startsWith('{') || content.startsWith('[')) {
    lang = 'json'
  } else if (content.startsWith('---')) {
    lang = 'yaml'
  } else if (content.startsWith("--")) {
    lang = "sql"
  }
  return lang
}

const addKey = () => {

}

const deleteKey = () => {
  for (let value of treeValue.value) {
    console.log("---> ", JSON.stringify(value))
  }
}

const treeSelected = ({id}: any) => {
  if (!treeSelectable.value) {
    let selectedKv = id as KeyValue
    _getKV(props.session?.id, selectedKv.key).then((kv) => {
      let language = tryParseFileNameToType(kv.key)
      if (!language) {
        language = tryFileContentToType(_decodeBytesToString(kv.value))
      }
      editorConfig.language = language
      currentKv.value = kv
      currentKvChanged.value = false
    }).catch(e => {
      _tipError(e)
      currentKv.value = undefined
    })
  }
}

const toggleTreeSelectable = () => {
  treeValue.value = []
  treeSelectable.value = !treeSelectable.value
}

const editorChange = () => {
  if (currentKv.value) {
    currentKvChanged.value = true
  }
}

const editorSave = () => {
  if (currentKv.value && currentKvChanged.value) {
    saveKV()
  }
}

const saveKV = () => {
  let kv = currentKv.value
  if (editorRef.value && kv) {
    let value = editorRef.value.readDataString()
    loadingStore.save = true
    _putKV(props.session?.id, kv.key, _encodeStringToBytes(value)).then(() => {
      currentKvChanged.value = false
    }).catch(e => {
      _tipError(e)
    }).finally(() => {
      loadingStore.save = false
    })
  }
}

const loadVersionDiff = () => {
  let kv = currentKv.value
  if (!kv) {
    return
  }
  if (kv.version <= 1) {
    _tipWarn('No multiple versions')
    return;
  }
  loadingStore.diff = true
  versionDiffInfo.key = kv.key
  versionDiffInfo.version = kv.version
  versionDiffInfo.createRevision = kv.createRevision
  versionDiffInfo.modRevision = kv.modRevision
  //  当前版本
  versionDiffInfo.B.version = versionDiffInfo.modRevision
  versionDiffInfo.B.content = _decodeBytesToString(kv!.value)

  let lang = tryParseFileNameToType(kv.key)
  if (!lang) {
    lang = tryFileContentToType(versionDiffInfo.B.content)
  }

  switch (lang) {
    case 'text':
      versionDiffInfo.language = 'plaintext'
      break
    case 'sql':
      versionDiffInfo.language = 'SQL'
      break
    case 'md':
      versionDiffInfo.language = 'Markdown'
      break
    default:
      versionDiffInfo.language = lang.substring(0, 1).toUpperCase() + lang.substring(1)
  }

  _getKVHistoryVersions(
      props.session?.id,
      kv.key,
      kv.createRevision,
      kv.modRevision
  ).then(versions => {
    //  倒序
    versionDiffInfo.versionHistory = versions

    //  上个版本
    versionDiffInfo.A.version = versions[1]
    loadDiff(versionDiffInfo.A)
  }).catch(e => {
    console.error(e)
    _tipError(e)
  }).finally(() => {
    loadingStore.diff = false
  })
}

const loadDiff = (info: DiffInfo) => {
  _getKVByVersion(props.session?.id, versionDiffInfo.key, info.version).then(data => {
    info.content = _decodeBytesToString(data.value)
  }).catch(e => {
    _tipWarn(`Failed to load revision ${info.version}: ${e}`)
    info.content = ''
  }).finally(() => {
    if (!versionDiffInfo.show) {
      versionDiffInfo.show = true
    }
  })
}

const versionSelectItemProps = (version: number) => {
  let item: Record<string, any> = {
    title: version,
    color: 'primary',
    density: 'compact'
  }

  if (version == versionDiffInfo.createRevision) {
    item.subtitle = 'create'
    item['append-icon'] = 'mdi-creation-outline'
  }
  if (version == versionDiffInfo.modRevision) {
    item.subtitle = 'latest'
    item['append-icon'] = 'mdi-new-box'
  }
  return item
}

</script>

<template>
  <div class="fill-height overflow-y-auto">
    <v-layout class="action-area">
      <v-btn class="text-none"
             prepend-icon="mdi-refresh"
             color="primary"
             @click="loadAllKeys"
      >Refresh
      </v-btn>
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
      <v-spacer></v-spacer>
      <v-tooltip v-if="session.namespace"
                 location="bottom"
                 text="The namespace of the current connection"
      >
        <template v-slot:activator="{ props }">
          <v-chip v-bind="props"
                  label
                  color="blue-grey-darken-1"
                  class="font-weight-bold"
                  prepend-icon="mdi-key"
          >{{ session.namespace }}
          </v-chip>
        </template>
      </v-tooltip>
      <v-tooltip v-if="currentKv"
                 location="bottom"
                 text="Current key"
      >
        <template v-slot:activator="{ props }">
          <v-chip v-bind="props"
                  label
                  color="primary"
                  class="font-weight-bold ml-2"
          >{{ currentKv.key }}
          </v-chip>
        </template>
      </v-tooltip>

    </v-layout>
    <v-layout class="main-area">
      <drag-box>
        <drag-item class="overflow-y-auto" style="min-width: 300px">
          <v-treeview
              v-model:selected="treeValue"
              :items="treeData"
              open-strategy="multiple"
              item-value="data"
              :selectable="treeSelectable"
              :select-strategy="treeSelectable ? 'leaf' : 'single-leaf'"
              @click:select="treeSelected"
              return-object
              open-on-click
              slim
              density="compact"
              class="user-select-none"
              height="100%"
          >
            <template v-slot:prepend="{ item }">
              <v-icon v-if="!item.file">mdi-folder</v-icon>
              <v-icon v-else>
                {{ fileIcon[item.iconKey] }}
              </v-icon>
            </template>
          </v-treeview>
        </drag-item>
        <drag-item style="width: calc(100% - 300px)" :show-resize-line="false">
          <editor ref="editorRef"
                  v-if="currentKv"
                  :key="currentKv.key"
                  :value="_decodeBytesToString(currentKv.value)"
                  :config="editorConfig"
                  @change="editorChange"
                  @save="editorSave">
            <template #headerPrepend>
              <div>
                <v-btn
                    color="primary"
                    size="small"
                    @click="saveKV"
                    :text="`Save${currentKvChanged ? ' *' : ''}`"
                    class="mr-2 text-none"
                    :loading="loadingStore.save"
                    prepend-icon="mdi-content-save-outline"
                ></v-btn>
                <v-btn
                    color="cyan-darken-1"
                    size="small"
                    @click="loadVersionDiff"
                    text="Version Diff"
                    class="mr-2 text-none"
                    :loading="loadingStore.diff"
                    prepend-icon="mdi-vector-difference"
                ></v-btn>
                <v-btn
                    color="light-green-darken-1"
                    size="small"
                    text="Copy And Save"
                    class="mr-2 text-none"
                    prepend-icon="mdi-content-copy"
                ></v-btn>
                <v-btn
                    color="deep-orange-darken-1"
                    size="small"
                    text="Delete"
                    class="mr-2 text-none"
                    prepend-icon="mdi-trash-can-outline"
                ></v-btn>
              </div>
            </template>
            <template #footerPrepend>
              <div>
                <span class="editor-footer-item"><strong>Version</strong>: {{ currentKv.version }}</span>
                <span class="editor-footer-item"><strong>Create Revision</strong>: {{ currentKv.createRevision }}</span>
                <span class="editor-footer-item"><strong>Modify Revision</strong>: {{ currentKv.modRevision }}</span>
                <span class="editor-footer-item"
                      v-if="currentKv.lease != '0'"><strong>Lease</strong>: {{ currentKv.lease }}</span>
              </div>
            </template>
          </editor>
          <div v-else class="no-key-preview">
            <v-empty-state icon="mdi-text-box-edit-outline"
                           headline="Please select a key"
                           title="KV Editor"
                           text="Select a key to view its details or edit it."
                           class="pt-12 pb-12"
            >

            </v-empty-state>
          </div>
        </drag-item>
      </drag-box>
    </v-layout>

    <!--    Diff  -->
    <v-dialog
        v-model="versionDiffInfo.show"
        persistent
        max-width="70vw"
        min-width="500px"
    >
      <v-card
          :min-width="800"
          :title="versionDiffInfo.key"
          :key="versionDiffInfo.key"
      >
        <template v-slot:prepend>
          <v-icon>mdi-vector-difference</v-icon>
        </template>
        <template v-slot:append>
          <v-icon class="cursor-pointer" @click="versionDiffInfo.show = false">mdi-close</v-icon>
        </template>
        <v-card-text>
          <v-layout class="pt-5">
            <v-select
                variant="outlined"
                v-model="versionDiffInfo.A.version"
                density="compact"
                :items="versionDiffInfo.versionHistory"
                :item-props="versionSelectItemProps"
                :width="10"
                hide-details
                persistent-hint
                class="mr-3"
                label="Version A"
                @update:model-value="loadDiff(versionDiffInfo.A)"
            ></v-select>

            <v-spacer></v-spacer>

            <v-select
                variant="outlined"
                v-model="versionDiffInfo.B.version"
                density="compact"
                :items="versionDiffInfo.versionHistory"
                :item-props="versionSelectItemProps"
                :width="10"
                hide-details
                persistent-hint
                class="mr-3"
                label="Version B"
                @update:model-value="loadDiff(versionDiffInfo.B)"
            ></v-select>
          </v-layout>

          <code-diff
              style="max-height: 70vh;min-height:50vh;"
              :old-string="versionDiffInfo.A.content"
              :filename="`Revision: ${versionDiffInfo.A.version}`"
              :new-string="versionDiffInfo.B.content"
              :new-filename="`Revision: ${versionDiffInfo.B.version}`"
              :theme="isDarkTheme ? 'dark' : 'light'"
              :language="versionDiffInfo.language"
              output-format="side-by-side"/>
        </v-card-text>
      </v-card>
    </v-dialog>

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
