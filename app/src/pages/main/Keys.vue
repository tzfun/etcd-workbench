<script setup lang="ts">

import {
  _deleteKV,
  _getAllKeys,
  _getAllKeysPaging,
  _getKV,
  _getKVByVersion,
  _getKVHistoryVersions,
  _handleError,
  _putKV,
  _putKVWithLease
} from "~/common/services.ts";
import {_confirm, _confirmSystem, _loading, _tipInfo, _tipSuccess, _tipWarn} from "~/common/events.ts";
import {computed, nextTick, onMounted, onUnmounted, PropType, reactive, ref} from "vue";
import {ErrorPayload, SessionData} from "~/common/transport/connection.ts";
import DragBox from "~/components/drag-area/DragBox.vue";
import DragItem from "~/components/drag-area/DragItem.vue";
import {KeyValue} from "~/common/transport/kv.ts";
import Editor from "~/components/editor/Editor.vue";
import {_decodeBytesToString, _isEmpty} from "~/common/utils.ts";
import {EditorConfig} from "~/common/types.ts";
import {CodeDiff} from "v-code-diff";
import {useTheme} from "vuetify";
import CountDownTimer from "~/components/CountDownTimer.vue";
import {_useSettings} from "~/common/store.ts";
import Tree from "~/components/tree/Tree.vue";

const theme = useTheme()

type DiffInfo = {
  version: number,
  content: string
}

const KEY_SPLITTER = computed<string>(() => {
  return _useSettings().value.kvPathSplitter
})

const LIMIT_PER_PAGE = computed(() => {
  return _useSettings().value.kvLimitPerPage
})

const props = defineProps({
  session: {
    type: Object as PropType<SessionData>,
    required: true
  }
})

const enforceLoadAllKey = ref<boolean>(false)
const kvTree = ref<InstanceType<typeof Tree>>()

const kvCount = ref<number>(0)
const currentKv = ref<KeyValue>()
const currentKvChanged = ref<boolean>(false)
const keyLeaseListeners = reactive<Set<any>>(new Set())
const paginationKeyCursor = ref<string | undefined>("")

const kvEditorContainerRef = ref()
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
  loadMore: false,
  getKey: false
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
  //  海量数据加载时会导致页面其他动画卡顿，这里延迟加载
  setTimeout(() => {
    nextTick(() => {
      _loading(true)
      refreshAllKeys().finally(() => {
        _loading(false)
      })
    })
  }, 200)
})

onUnmounted(() => {
  clearAllKeyLeaseListener()
})

const refreshAllKeys = (): Promise<any> => {
  currentKv.value = undefined
  kvCount.value = 0
  clearAllKeyLeaseListener()
  kvTree.value?.rerender()

  if (_useSettings().value.kvPaginationQuery && !enforceLoadAllKey.value) {
    paginationKeyCursor.value = ""
    return loadNextPage()
  } else {
    return loadAllKeys()
  }
}

const loadAllKeys = (): Promise<any> => {
  paginationKeyCursor.value = undefined
  loadingStore.loadMore = true
  return _getAllKeys(props.session?.id).then(data => {
    kvCount.value += data.length
    addDataListToTree(data)
  }).catch(e => {
    _handleError({
      e,
      session: props.session
    })
  }).finally(() => {
    loadingStore.loadMore = false
  })
}

const loadNextPage = (): Promise<any> => {
  let cursor = paginationKeyCursor.value
  if (cursor != undefined) {
    loadingStore.loadMore = true
    let limit: number = LIMIT_PER_PAGE.value as number
    return _getAllKeysPaging(props.session?.id, cursor, limit).then((data: KeyValue[]) => {
      kvCount.value += data.length
      if (data.length < limit) {
        paginationKeyCursor.value = undefined
      }

      if (data.length > 0) {
        if (paginationKeyCursor.value != undefined) {
          paginationKeyCursor.value = data[data.length - 1].key
        }

        addDataListToTree(data)
      }
    }).catch((e: ErrorPayload | string) => {
      _handleError({
        e,
        session: props.session
      })
    }).finally(() => {
      loadingStore.loadMore = false
    })
  } else {
    return Promise.resolve()
  }
}

const removeKeysFromTree = (keys: string[]) => {
  for (let key of keys) {
    kvTree.value?.removeItemFromTree(key)
    if (currentKv.value && currentKv.value.key == key) {
      currentKv.value = undefined
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

    kvTree.value?.addItemToTree(key)
  }).catch(e => {
    _handleError({
      e,
      session: props.session
    })
  }).finally(() => {
    loadingStore.confirmNewKey = false
  })
}

const addDataListToTree = (data: KeyValue[]) => {
  for (let kv of data) {
    kvTree.value?.addItemToTree(kv.key)
  }
}

const deleteKeyBatch = () => {
  let keys: string[] = kvTree.value!.getSelectedItems()
  if (keys.length == 0) {
    _tipInfo('Please select at least one key')
    return
  }

  let containsCurrentKV = currentKv.value && keys.includes(currentKv.value.key)
  let message = "Please confirm to permanently delete these keys:<br/><br/><strong>"
  const showCount = 20
  if (keys.length >= showCount) {
    message += keys.slice(0, showCount).join('<br/>')
    message += `<br/><br/> ... Omit ${keys.length - showCount} keys`
  } else {
    message += keys.join('<br/>')
  }
  message += '</strong>'
  _confirmSystem(message).then(() => {
    _loading(true, "Deleting keys...")
    loadingStore.deleteBatch = true
    _deleteKV(props.session?.id, keys).then(() => {
      if (containsCurrentKV) {
        currentKv.value = undefined
      }
      removeKeysFromTree(keys)
    }).catch(e => {
      _handleError({
        e,
        session: props.session
      })
    }).finally(() => {
      loadingStore.deleteBatch = false
      _loading(false)
    })
  }).catch(() => {
  })
}

const onClickTreeItem = (key: string) => {
  loadingStore.getKey = true
  _getKV(props.session?.id, key).then((kv) => {
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
      removeKeysFromTree([key])
    }
    _handleError({
      e,
      session: props.session
    })
    currentKv.value = undefined
  }).finally(() => {
    loadingStore.getKey = false
  })
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
    let doSave = () => {
      let value: number[] = editorRef.value!.readDataBytes()
      loadingStore.save = true
      _putKV(props.session?.id, kv!.key, value).then(() => {
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

    if (_useSettings().value.kvCheckFormatBeforeSave) {
      editorRef.value.tryFormatContent().then(() => {
        doSave()
      }).catch(() => {
        _confirm("Warning", "The format checker found incorrect content. Do you want to submit it anyway?").then(() => {
          doSave()
        }).catch(() => {
        })
      })
    } else {
      doSave()
    }
  }
}

const loadVersionDiff = () => {

  let key = currentKv.value ? currentKv.value.key : null
  if (!key) {
    return
  }

  loadingStore.diff = true
  versionDiffInfo.key = key
  _getKV(props.session?.id, key).then(dataB => {
    versionDiffInfo.version = dataB.version
    versionDiffInfo.createRevision = dataB.createRevision
    versionDiffInfo.modRevision = dataB.modRevision

    //  当前版本
    versionDiffInfo.B.version = versionDiffInfo.modRevision
    versionDiffInfo.B.content = _decodeBytesToString(dataB!.value)

    let lang = tryParseFileNameToType(dataB.key)
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
        key!,
        dataB.createRevision,
        dataB.modRevision
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

  }).catch(e => {
    _handleError({
      e,
      session: props.session
    })
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
      removeKeysFromTree(keys)
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
  removeKeysFromTree([key])
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
    <v-layout class="action-area pa-5">
      <v-btn class="text-none"
             prepend-icon="mdi-refresh"
             variant="outlined"
             @click="refreshAllKeys"
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
                  color="brown-lighten-2"
                  class="font-weight-bold"
                  prepend-icon="mdi-view-headline"
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
                  color="light-blue-accent-4"
                  class="font-weight-bold ml-2"
          >{{ currentKv.key }}
          </v-chip>
        </template>
      </v-tooltip>
    </v-layout>
    <v-layout class="main-area">
      <drag-box>
        <drag-item class="overflow-y-auto" style="min-width: 300px">

          <Tree ref="kvTree"
                :tree-id="`kv-tree-${new Date().getTime()}`"
                :key-splitter="KEY_SPLITTER"
                style="height: calc(100% - 30px);"
                @on-click="onClickTreeItem"
          ></Tree>
          <v-sheet class="loadMoreArea d-flex align-center justify-center loadMoreArea"
          >
            <v-btn
                v-if="paginationKeyCursor != undefined"
                block
                density="compact"
                color="cyan-darken-4"
                class="text-none border-none user-select-none"
                style="border-radius: 0;"
                text="Load More"
                @click="loadNextPage"
                prepend-icon="mdi-book-open-page-variant-outline"
            >
              <template #append>
                <span class="count  user-select-none" title="The number of keys loaded">({{kvCount}})</span>
              </template>
            </v-btn>
            <p v-else class="count text-center text-medium-emphasis user-select-none" title="The number of keys loaded">Loaded {{kvCount}} keys</p>

          </v-sheet>
        </drag-item>
        <drag-item ref="kvEditorContainerRef"
                   style="width: calc(100% - 300px)"
                   :show-resize-line="false">
          <v-overlay
              v-model="loadingStore.getKey"
              persistent
              contained
              class="align-center justify-center ma-0"
              :z-index="100"
          >
            <v-progress-circular
                color="primary"
                size="40"
                indeterminate
            ></v-progress-circular>
          </v-overlay>

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
                           class="mx-auto my-auto user-select-none"
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
        max-width="80vw"
        min-width="500px"
        scrollable
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
              style="max-height: 60vh;min-height: 40vh;"
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
$--action-area-height: 60px;
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

.loadMoreArea {
  $--load-more-area-height: 30px;
  height: $--load-more-area-height;

  .count {
    font-size: 0.8em;
    cursor: default;
    height: $--load-more-area-height;
    line-height: $--load-more-area-height;
  }
}
</style>
