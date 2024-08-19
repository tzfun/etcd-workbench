<script setup lang="ts">

import {
  _deleteKV,
  _getAllKeysPaging,
  _getKV,
  _getKVByVersion,
  _getKVHistoryVersions,
  _handleError,
  _putKV,
  _putKVWithLease
} from "~/common/services.ts";
import {_confirmSystem, _tipInfo, _tipSuccess, _tipWarn} from "~/common/events.ts";
import {computed, onMounted, onUnmounted, PropType, reactive, ref} from "vue";
import {SessionData} from "~/common/transport/connection.ts";
import DragBox from "~/components/DragBox.vue";
import DragItem from "~/components/DragItem.vue";
import {KeyValue} from "~/common/transport/kv.ts";
import Editor from "~/components/editor/Editor.vue";
import {_decodeBytesToString, _isEmpty, fileTypeIcon} from "~/common/utils.ts";
import {EditorConfig} from "~/common/types.ts";
import {CodeDiff} from "v-code-diff";
import {useTheme} from "vuetify";
import CountDownTimer from "~/components/CountDownTimer.vue";

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
const LIMIT_PER_PAGE = 2

const props = defineProps({
  session: {
    type: Object as PropType<SessionData>,
    required: true
  }
})

const treeValue = ref<KeyValue[]>([])
const treeData = reactive<TreeNode>({
  title: 'root',
  file: false,
  iconKey: 'dir',
  children: []
})
const treeSelectable = ref(false)
const currentKv = ref<KeyValue>()
const currentKvChanged = ref<boolean>(false)
const keyLeaseListeners = reactive<Set<any>>(new Set())
const paginationKeyCursor = ref<string | undefined>("")

const editorRef = ref<InstanceType<typeof Editor>>()
const newKeyEditorRef = ref<InstanceType<typeof Editor>>()
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
  diff: false,
  delete: false,
  deleteBatch: false,
  confirmNewKey: false,
  loadMore: false
})

const newKeyDialog = reactive({
  show: false,
  title: 'New Key',
  copyAndSave: false,
  fromKey: '',
  value: '',
  key: '',
  ttl: '',
  lease: '',
  model: <'none' | 'ttl' | 'lease'>'none'
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

onUnmounted(() => {
  clearAllKeyLeaseListener()
})

const loadAllKeys = () => {
  paginationKeyCursor.value = ""
  treeData.children = []
  clearAllKeyLeaseListener()

  loadNextPage()
}

const loadNextPage = () => {
  let cursor = paginationKeyCursor.value
  if (cursor != undefined) {
    loadingStore.loadMore = true
    _getAllKeysPaging(props.session?.id, cursor, LIMIT_PER_PAGE).then(data => {
      if (data.length == 0) {
        paginationKeyCursor.value = undefined
      } else {
        paginationKeyCursor.value = data[data.length - 1].key
        addKvListToTree(data)
      }
      console.log(data)
    }).catch(e => {
      _handleError({
        e,
        session: props.session
      })
    }).finally(() => {
      loadingStore.loadMore = false
    })
  }
}

const addKvListToTree = (data: KeyValue[]) => {
  for (let kv of data) {
    addKvToTree(kv)
  }
}

const addKvToTree = (kv: KeyValue) => {
  let key = kv.key
  //  为了方便解析为统一的树状结构，如果key不是以分隔符开头，默认补充分隔符
  if (!key.startsWith(KEY_SPLITTER)) {
    key = KEY_SPLITTER + key
  }

  let splits = key.split(KEY_SPLITTER)
  let node: TreeNode = treeData

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

const removeKeyFromTreeData = (keys: string[]) => {
  keysLoop:
      for (let key of keys) {
        //  为了方便解析为统一的树状结构，如果key不是以分隔符开头，默认补充分隔符
        if (!key.startsWith(KEY_SPLITTER)) {
          key = KEY_SPLITTER + key
        }
        let pathArr = key.split(KEY_SPLITTER)
        let stack: TreeNode[] = []

        let nodeArr: TreeNode[] = treeData.children!

        //  搜索前缀路径
        keyPathLoop:
            for (let i = 1; i < pathArr.length - 1; i++) {
              let path = pathArr[i]
              for (let node of nodeArr) {
                if (node.title === path && !node.file) {
                  stack.push(node)
                  nodeArr = node.children ? node.children : []
                  continue keyPathLoop;
                }
              }
              continue keysLoop
            }

        let path = pathArr[pathArr.length - 1]
        //  第一层
        if (stack.length == 0) {
          let idx = -1
          for (let i = 0; i < nodeArr.length; i++) {
            let node = nodeArr[i]
            if (node.title === path && node.file) {
              idx = i
              break
            }
          }
          if (idx >= 0) {
            nodeArr.splice(idx, 1)
          }
          continue
        }

        let removedFile = false
        let needRemoveDirNode: TreeNode | null = null
        while (true) {
          let node = stack.pop()
          if (removedFile) {
            //  文件已删除，开始清空空目录
            if (node) {
              let idx = node.children!.indexOf(needRemoveDirNode!)
              if (idx >= 0) {
                node.children?.splice(idx, 1)
              }
            } else {
              let idx = treeData.children!.indexOf(needRemoveDirNode!)
              if (idx >= 0) {
                treeData.children!.splice(idx, 1)
              }
              break
            }
          } else {
            //  删除目标文件
            let nodeArr: TreeNode[]
            if (node) {
              nodeArr = node.children ? node.children : []
            } else {
              nodeArr = treeData.children!
            }

            let idx = -1
            for (let i = 0; i < nodeArr.length; i++) {
              let node = nodeArr[i]
              if (node.title === path && node.file) {
                idx = i
                break
              }
            }
            if (idx >= 0) {
              nodeArr.splice(idx, 1)
              removedFile = true
              if (nodeArr.length > 0) {
                break
              }
              if (node) {
                needRemoveDirNode = node
              } else {
                break
              }
            } else {
              //  未找到
              break
            }
          }
        }
      }
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

const showNewKeyDialog = () => {
  newKeyDialog.key = ''
  newKeyDialog.ttl = ''
  newKeyDialog.lease = ''
  newKeyDialog.value = ''
  newKeyDialog.fromKey = ''
  newKeyDialog.model = 'none'
  newKeyDialog.title = 'New Key'
  newKeyDialog.copyAndSave = false
  newKeyDialog.show = true
}

const showCopyAndSaveDialog = (fromKey: string, fromValue: string) => {
  newKeyDialog.key = ''
  newKeyDialog.ttl = ''
  newKeyDialog.lease = ''
  newKeyDialog.model = 'none'
  newKeyDialog.fromKey = fromKey
  newKeyDialog.value = fromValue
  newKeyDialog.title = 'Copy And Save'
  newKeyDialog.copyAndSave = true
  newKeyDialog.show = true
}

const putKey = () => {
  if (_isEmpty(newKeyDialog.key)) {
    _tipWarn("Key can not be empty")
    return
  }
  if (newKeyDialog.model === 'ttl' && _isEmpty(newKeyDialog.ttl)) {
    _tipWarn("Please input a valid ttl")
    return
  }
  if (newKeyDialog.model === 'lease' && _isEmpty(newKeyDialog.lease)) {
    _tipWarn("Please input a valid lease id")
    return
  }
  let key = newKeyDialog.key
  let value: number[] = newKeyEditorRef.value!.readDataBytes()
  let promise: Promise<undefined>
  if (newKeyDialog.model === 'lease') {
    promise = _putKVWithLease(props.session?.id, key, value, newKeyDialog.lease)
  } else {
    let ttl = newKeyDialog.model === 'none' ? undefined : parseInt(newKeyDialog.ttl)
    promise = _putKV(props.session?.id, key, value, ttl)
  }

  loadingStore.confirmNewKey = true
  promise.then(() => {
    _tipSuccess("Succeeded!")
    newKeyDialog.show = false

    //  @ts-ignore
    let kv: KeyValue = {
      key: key,
      value: []
    }
    addKvToTree(kv)
  }).catch(e => {
    _handleError({
      e,
      session: props.session
    })
  }).finally(() => {
    loadingStore.confirmNewKey = false
  })
}

const deleteKeyBatch = () => {
  if (treeValue.value.length == 0) {
    _tipInfo('Please select at least one key')
    return
  }
  let keys: string[] = []
  let containsCurrentKV = false
  for (let value of treeValue.value) {
    keys.push(value.key)
    if (currentKv.value && currentKv.value.key == value.key) {
      containsCurrentKV = true
    }
  }

  _confirmSystem(`Please confirm to permanently delete these keys: <br/><br/><strong>${keys.join('<br/>')}</strong>`).then(() => {
    loadingStore.deleteBatch = true
    _deleteKV(props.session?.id, keys).then(() => {
      if (containsCurrentKV) {
        currentKv.value = undefined
      }
      removeKeyFromTreeData(keys)
    }).catch(e => {
      _handleError({
        e,
        session: props.session
      })
    }).finally(() => {
      loadingStore.deleteBatch = false
    })
  }).catch(() => {
  })
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

      if (kv.leaseInfo) {
        let timer = setTimeout(() => {
          keyLeaseListeners.delete(timer)
          onKeyTimeOver(kv.key)
        }, kv.leaseInfo.ttl * 1000)
        keyLeaseListeners.add(timer)
      }

    }).catch(e => {
      if (e.errType && e.errType == 'ResourceNotExist') {
        removeKeyFromTreeData([selectedKv.key])
      }
      _handleError({
        e,
        session: props.session
      })
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
    let value: number[] = editorRef.value.readDataBytes()
    loadingStore.save = true
    _putKV(props.session?.id, kv.key, value).then(() => {
      currentKvChanged.value = false
    }).catch(e => {
      _handleError({
        e,
        session: props.session
      })
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
    if (versions.length < 2) {
      _tipWarn('No multiple versions, required revision has been compacted')
      return;
    }

    //  倒序
    versionDiffInfo.versionHistory = versions

    //  上个版本
    versionDiffInfo.A.version = versions[1]
    loadDiff(versionDiffInfo.A)
  }).catch(e => {
    _handleError({
      e,
      session: props.session
    })
  }).finally(() => {
    loadingStore.diff = false
  })
}

const loadDiff = (info: DiffInfo) => {
  _getKVByVersion(props.session?.id, versionDiffInfo.key, info.version).then(data => {
    info.content = _decodeBytesToString(data.value)
    if (!versionDiffInfo.show) {
      versionDiffInfo.show = true
    }
  }).catch(e => {
    _handleError({
      e,
      prefix: `Failed to load revision ${info.version}: `,
      session: props.session
    })
    info.content = ''
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

const deleteKey = () => {
  if (!currentKv.value) {
    return
  }
  let key = currentKv.value.key
  _confirmSystem(`Please confirm to permanently delete key: <strong>${key}</strong>`).then(() => {
    loadingStore.delete = true
    let keys = [key]
    _deleteKV(props.session?.id, [key]).then(() => {
      currentKv.value = undefined
      removeKeyFromTreeData(keys)
    }).catch(e => {
      _handleError({
        e,
        session: props.session
      })
    }).finally(() => {
      loadingStore.delete = false
    })
  }).catch(() => {
  })
}

const onKeyTimeOver = (key: string) => {
  if (currentKv.value && currentKv.value.key) {
    currentKv.value = undefined
  }
  removeKeyFromTreeData([key])
}

const clearAllKeyLeaseListener = () => {
  for (let keyLeaseListener of keyLeaseListeners) {
    clearTimeout(keyLeaseListener)
  }

  keyLeaseListeners.clear()
}

</script>

<template>
  <div class="fill-height overflow-y-auto">
    <v-layout class="action-area">
      <v-btn class="text-none"
             prepend-icon="mdi-refresh"
             variant="outlined"
             @click="loadAllKeys"
             :loading="loadingStore.loadMore"
             text="Refresh"
      ></v-btn>
      <v-btn class="text-none ml-2"
             prepend-icon="mdi-file-document-plus-outline"
             color="green"
             @click="showNewKeyDialog"
             text="Add Key"
      ></v-btn>
      <v-btn class="text-none ml-2"
             :prepend-icon="treeSelectable ? 'mdi-checkbox-outline' : 'mdi-checkbox-blank-outline'"
             color="secondary"
             @click="toggleTreeSelectable"
             text="Select Keys"
      ></v-btn>
      <v-btn class="text-none ml-2"
             v-show="treeSelectable"
             prepend-icon="mdi-file-document-minus-outline"
             color="red"
             @click="deleteKeyBatch"
             :loading="loadingStore.deleteBatch"
             text="Delete Keys"
      ></v-btn>
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
              :items="treeData.children"
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
              height="calc(100% - 30px)"
          >
            <template v-slot:prepend="{ item }">
              <v-icon v-if="!item.file">mdi-folder</v-icon>
              <v-icon v-else>
                {{ fileTypeIcon[item.iconKey] }}
              </v-icon>
            </template>
          </v-treeview>
          <v-sheet height="30px"
                   class="d-flex align-center justify-center"
          >
            <v-btn
                v-if="paginationKeyCursor != undefined"
                block
                density="compact"
                color="secondary"
                class="text-none border-none"
                style="border-radius: 0;"
                text="Load More"
                @click="loadNextPage"
            ></v-btn>
          </v-sheet>
        </drag-item>
        <drag-item style="width: calc(100% - 300px)" :show-resize-line="false">
          <div v-if="currentKv" class="fill-height">
            <v-layout class="editor-header">
              <v-spacer></v-spacer>
              <v-btn
                  v-show="currentKvChanged"
                  color="primary"
                  size="small"
                  @click="saveKV"
                  text="Save"
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
                  @click="showCopyAndSaveDialog(currentKv.key, _decodeBytesToString(currentKv.value))"
              ></v-btn>
              <v-btn
                  color="deep-orange-darken-1"
                  size="small"
                  @click="deleteKey"
                  :loading="loadingStore.delete"
                  text="Delete"
                  class="mr-2 text-none"
                  prepend-icon="mdi-trash-can-outline"
              ></v-btn>
            </v-layout>

            <div class="editor-body">
              <editor ref="editorRef"
                      :key="currentKv.key"
                      :value="_decodeBytesToString(currentKv.value)"
                      :config="editorConfig"
                      @change="editorChange"
                      @save="editorSave">
                <template #footer>
                  <span class="editor-footer-item ml-0" v-if="currentKv.leaseInfo">
                    <v-tooltip location="top"
                               :text="`Granted TTL: ${currentKv.leaseInfo.grantedTtl} s`">
                      <template v-slot:activator="{ props }">
                        <span class="text-secondary user-select-none"
                              v-bind="props">
                          <CountDownTimer :value="currentKv.leaseInfo.ttl"></CountDownTimer>
                        </span>
                      </template>
                    </v-tooltip>
                  </span>
                  <v-spacer></v-spacer>
                  <span class="editor-footer-item"><strong>Version</strong>: {{ currentKv.version }}</span>
                  <span class="editor-footer-item"><strong>Create Revision</strong>: {{
                      currentKv.createRevision
                    }}</span>
                  <span class="editor-footer-item"><strong>Modify Revision</strong>: {{ currentKv.modRevision }}</span>
                  <span class="editor-footer-item"
                        v-if="currentKv.lease != '0'"><strong>Lease</strong>: {{ currentKv.lease }}</span>
                </template>
              </editor>
            </div>
          </div>

          <div v-else class="no-key-preview fill-height">
            <v-empty-state icon="mdi-text-box-edit-outline"
                           headline="Please select a key"
                           title="Select a key to view its details or edit it"
                           class="mx-auto my-auto"
            >

            </v-empty-state>
          </div>
        </drag-item>
      </drag-box>
    </v-layout>

    <!--    Diff弹窗  -->
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

    <!--  Add Key弹窗-->
    <v-dialog
        v-model="newKeyDialog.show"
        persistent
        max-width="70vw"
        min-width="500px"
        scrollable
    >
      <v-card :title="newKeyDialog.title">
        <v-card-text>
          <v-layout class="mb-5" v-show="newKeyDialog.copyAndSave">
            <span class="new-key-form-label">From Key: </span>
            <v-text-field v-model="newKeyDialog.fromKey"
                          density="comfortable"
                          prepend-inner-icon="mdi-key"
                          :prefix="session.namespace"
                          hide-details
                          readonly
            ></v-text-field>
          </v-layout>
          <v-layout class="mb-5">
            <span class="new-key-form-label">Key: </span>
            <v-text-field v-model="newKeyDialog.key"
                          density="comfortable"
                          prepend-inner-icon="mdi-key"
                          :prefix="session.namespace"
                          hint="The key under namespace (if it exists)"
                          persistent-hint
            ></v-text-field>
          </v-layout>
          <v-layout class="mb-5">
            <span class="new-key-form-label"></span>
            <v-radio-group
                v-model="newKeyDialog.model"
                inline
                hide-details
            >
              <v-radio
                  label="Never Expire"
                  value="none"
              ></v-radio>
              <v-radio
                  label="With TTL"
                  value="ttl"
              ></v-radio>
              <v-radio
                  label="With Lease"
                  value="lease"
              ></v-radio>
            </v-radio-group>
          </v-layout>
          <v-layout class="mb-5" v-if="newKeyDialog.model == 'ttl'">
            <span class="new-key-form-label">TTL(s): </span>
            <v-text-field v-model="newKeyDialog.ttl"
                          type="number"
                          density="comfortable"
                          prepend-inner-icon="mdi-clock-time-eight"
                          hint="The key expiration time in seconds, optional. If left blank, the key will never expire."
                          persistent-hint
            ></v-text-field>
          </v-layout>
          <v-layout class="mb-5" v-if="newKeyDialog.model == 'lease'">
            <span class="new-key-form-label">Lease: </span>
            <v-text-field v-model="newKeyDialog.lease"
                          type="number"
                          density="comfortable"
                          prepend-inner-icon="mdi-identifier"
                          hint="Bind the key to this lease, they share the same lifecycle. Please make sure the lease already exists, otherwise the operation will fail."
                          persistent-hint
            ></v-text-field>
          </v-layout>
          <div style="height: 50vh;width:100%">
            <editor ref="newKeyEditorRef"
                    :value="newKeyDialog.value"
                    :config="editorConfig"></editor>
          </div>
        </v-card-text>
        <v-card-actions>
          <v-btn text="Cancel"
                 variant="text"
                 class="text-none"
                 @click="newKeyDialog.show = false"
          ></v-btn>

          <v-btn text="Confirm"
                 variant="flat"
                 class="text-none"
                 color="primary"
                 @click="putKey"
                 :loading="loadingStore.confirmNewKey"
          ></v-btn>
        </v-card-actions>
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

  $--editor-header-height: 40px;

  .editor-header {
    height: $--editor-header-height;
  }

  .editor-body {
    height: calc(100% - $--editor-header-height);
  }

}

.new-key-form-label {
  display: inline-block;
  width: 80px;
  line-height: 48px;
}
</style>
