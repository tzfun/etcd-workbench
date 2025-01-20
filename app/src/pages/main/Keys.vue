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
  _putKVWithLease,
  _searchByPrefix,
  _updateKeyCollection
} from "~/common/services.ts";
import {
  _confirm,
  _confirmSystem,
  _copyToClipboard,
  _emitLocal,
  _listenLocal,
  _loading,
  _tipInfo,
  _tipSuccess,
  _tipWarn,
  EventName
} from "~/common/events.ts";
import {computed, nextTick, onMounted, onUnmounted, PropType, reactive, ref} from "vue";
import {ErrorPayload, KeyMonitorConfig, SessionData} from "~/common/transport/connection.ts";
import DragBox from "~/components/drag-area/DragBox.vue";
import DragItem from "~/components/drag-area/DragItem.vue";
import {KeyValue} from "~/common/transport/kv.ts";
import Editor from "~/components/editor/Editor.vue";
import {_decodeBytesToString, _isEmpty, _tryParseDiffLanguage, _tryParseEditorLanguage} from "~/common/utils.ts";
import {EditorConfig, EditorHighlightLanguage} from "~/common/types.ts";
import {CodeDiff} from "v-code-diff";
import {useTheme} from "vuetify";
import CountDownTimer from "~/components/CountDownTimer.vue";
import {_saveGlobalStore, _useGlobalStore, _useSettings} from "~/common/store.ts";
import Tree from "~/components/tree/Tree.vue";
import {_isMac} from "~/common/windows.ts";
import { _debounce } from "~/common/utils";
import { SearchResult } from "~/common/transport/kv";
import { VTextField } from "vuetify/components";

const theme = useTheme()

type DiffInfo = {
  version: number,
  content: string
}

//  自动移除lease失效key的开关
const AUTO_REMOVE_EXPIRED_KEY = false

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
const kvCollectionTree = ref<InstanceType<typeof Tree>>()
const collectionDialog = ref<boolean>(false)
const addCollectionKeyForm = ref<string>("")

const kvCount = ref<number>(0)
const currentKv = ref<KeyValue>()
const currentKvChanged = ref<boolean>(false)
const showFormattedValue = ref<boolean>(false)
const keyLeaseListeners = reactive<Set<any>>(new Set())
const paginationKeyCursor = ref<string | undefined>("")
const editorAlert = reactive({
  enable: false,
  show: true,
  type: ''
})

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
  },
  useFormattedValue: false,
})

const searchDialog = reactive({
  show: false,
  inputValue: "",
  searchResult: <SearchResult | null>null,
  loading: false,
})

const isDarkTheme = computed<boolean>(() => {
  return theme.global.name.value === 'dark'
})

const editorContent = computed<string>(() => {
  if (currentKv.value) {
    if (currentKv.value.formattedValue) {
      if (showFormattedValue.value) {
        return currentKv.value.formattedValue!.value
      } else {
        return _decodeBytesToString(currentKv.value.value)
      }
    } else {
      return _decodeBytesToString(currentKv.value.value)
    }
  }
  return ""
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

  _listenLocal(EventName.KEY_MONITOR_CONFIG_CHANGE, e => {
    if (e.session == props.session?.id) {
      let key = e.key as string
      kvTree.value?.refreshDiyDom(key)
    }
  })
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

const addDataListToTree = (data: KeyValue[], ignoreIfExist?: boolean) => {
  kvCount.value += data.length
  for (let kv of data) {
    kvTree.value?.addItemToTree(kv.key, ignoreIfExist)
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

const showKV = (key: string): Promise<void> => {
  return new Promise((resolve, reject) => {
    loadingStore.getKey = true
    _getKV(props.session?.id, key).then((kv) => {
      resolve()

      editorConfig.language = _tryParseEditorLanguage(kv.key, kv.value, kv.formattedValue, props.session?.namespace)
      editorConfig.disabled = kv.formattedValue != undefined;
      editorAlert.enable = kv.formattedValue != undefined;
      editorAlert.type = kv.formattedValue == undefined ? '' : kv.formattedValue.source

      currentKv.value = kv
      currentKvChanged.value = false
      showFormattedValue.value = true

      if (kv.leaseInfo && AUTO_REMOVE_EXPIRED_KEY) {
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
      reject(e)
    }).finally(() => {
      loadingStore.getKey = false
    })
  })
}

const showKVUnwrapped = (key: string) => {
  showKV(key).then(() => {
  }).catch(e => {
    console.error(e)
  })
}

const addCollectionKey = (key: string) => {
  if (_isEmpty(key)) {
    return
  }
  let set = props.session!.keyCollectionSet!
  if (set.has(key)) {
    return
  }

  let collections = props.session!.keyCollection!;
  collections.push(key)

  _updateKeyCollection(props.session?.id, collections).then(() => {
    set.add(key)
    kvCollectionTree.value?.addItemToTree(key)
    kvTree.value?.refreshDiyDom(key)
  }).catch(e => {
    collections.pop()
    _handleError({
      e
    })
  })
}

const removeCollectionKey = (key: string) => {
  let collections = props.session!.keyCollection!
  let idx = collections.indexOf(key)
  if (idx >= 0) {
    collections.splice(idx, 1)
  }
  _updateKeyCollection(props.session?.id, collections).then(() => {
    props.session!.keyCollectionSet!.delete(key)
    kvCollectionTree.value?.removeItemFromTree(key)
    kvTree.value?.refreshDiyDom(key)
  }).catch(e => {
    collections.push(key)
    _handleError({
      e
    })
  })
}

const editorChange = ({modified}: { data: string, modified: boolean }) => {
  if (currentKv.value) {
    currentKvChanged.value = modified
  }
}

const editorChangeLanguage = (lang: EditorHighlightLanguage) => {
  if (currentKv.value) {
    let namespace = props.session!.namespace
    let fullKey = (namespace ? namespace : "") + currentKv.value.key
    let store = _useGlobalStore().value
    //  保存用户变更的文件格式类型
    let existFormat = store.fileFormatLogMap[fullKey]
    store.fileFormatLogMap[fullKey] = lang
    //  已存在记录，只需修改
    if (existFormat) {
      for (let i = store.fileFormatLog.length - 1; i >= 0; i--) {
        let fileFormat = store.fileFormatLog[i]
        if (fileFormat.key == fullKey) {
          fileFormat.format = lang
          break
        }
      }
    } else {
      let len = store.fileFormatLog.push({
        key: fullKey,
        format: lang
      })
      //  最多保存100条记录
      const LOG_LIMIT_SIZE = 100
      //  缓冲20条
      if (len > LOG_LIMIT_SIZE + 20) {
        let removed = store.fileFormatLog.splice(0, len - LOG_LIMIT_SIZE)
        for (let fileFormat of removed) {
          delete store.fileFormatLogMap[fileFormat.key]
        }
      }
    }

    _saveGlobalStore(store)
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
    if(dataB!.formattedValue) {
      versionDiffInfo.B.content = dataB.formattedValue.value
      versionDiffInfo.language = dataB.formattedValue.language as EditorHighlightLanguage
      versionDiffInfo.useFormattedValue = true
    } else {
      versionDiffInfo.B.content = _decodeBytesToString(dataB!.value)
      let lang = _tryParseEditorLanguage(dataB.key, versionDiffInfo.B.content, dataB.formattedValue, props.session?.namespace)
      versionDiffInfo.language = _tryParseDiffLanguage(lang)
      versionDiffInfo.useFormattedValue = false
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
    info.content = data.formattedValue ? data.formattedValue.value : _decodeBytesToString(data.value)

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
  if (AUTO_REMOVE_EXPIRED_KEY) {
    for (let keyLeaseListener of keyLeaseListeners) {
      clearTimeout(keyLeaseListener)
    }

    keyLeaseListeners.clear()
  }
}

const onClickKeyCollectionTreeItem = (key: string) => {
  kvTree.value?.addItemToTree(key, true)
  kvTree.value?.selectItem(key)
  showKV(key).then(() => {
    collectionDialog.value = false
  }).catch(() => {
  })
}

const editKeyMonitor = (key: string) => {
  let monitor: KeyMonitorConfig = props.session?.keyMonitorMap![key]

  if (monitor) {
    _emitLocal(EventName.EDIT_KEY_MONITOR, {
      session: props.session?.id,
      edit: true,
      monitor
    })
  }
}

const addKeyMonitor = (key: string) => {
  _emitLocal(EventName.EDIT_KEY_MONITOR, {
    session: props.session?.id,
    edit: false,
    key
  })
}

const openSearchDialog = () => {
  searchDialog.inputValue = ''
  searchDialog.searchResult = null
  searchDialog.show = true
  searchDialog.loading = false

}

const selectSearchItem = (kv: KeyValue) => {
  showKV(kv.key)
  kvTree.value?.selectItem(kv.key)
  searchDialog.show = false
}

const searchFromServer = _debounce(() => {
  if(_isEmpty(searchDialog.inputValue)) {
    searchDialog.searchResult = null
    return
  }
  searchDialog.loading = true
  _searchByPrefix(props.session?.id, searchDialog.inputValue).then((data: SearchResult) => {
    searchDialog.searchResult = data

    if(data) {
      addDataListToTree(data.results, true)
    }
  }).finally(() => {
    searchDialog.loading = false
  })
}, 1000)

</script>

<template>
  <div class="fill-height overflow-y-auto">
    <v-layout class="action-area pa-5">
      <v-btn 
            v-bind="props"
            variant="tonal"
            size="small"
            icon="mdi-refresh"
            @click="refreshAllKeys"
            :loading="loadingStore.loadMore"
            title="Refresh"
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

      <v-btn class="text-none ml-2"
             prepend-icon="mdi-star"
             color="yellow"
             @click="collectionDialog = true"
             text="My Collections"
      ></v-btn>

      <v-btn class="text-none ml-2"
            v-bind="props"
            prepend-icon="mdi-text-box-search-outline"
            color="blue-lighten-1"
            @click="openSearchDialog"
            text="Search"
            title="Search from etcd server"
      ></v-btn>

      <v-spacer></v-spacer>

      <v-tooltip v-if="session.namespace"
                 location="top"
                 text="Namespace"
      >
        <template v-slot:activator="{ props }">
          <v-chip v-bind="props"
                  label
                  color="brown-lighten-2"
                  class="font-weight-bold"
                  prepend-icon="mdi-home"
                  @click="_copyToClipboard(session.namespace)"
          >{{ session.namespace }}
          </v-chip>
        </template>
      </v-tooltip>
      <v-tooltip v-if="currentKv"
                 location="top"
                 text="Current key"
      >
        <template v-slot:activator="{ props }">
          <v-chip v-bind="props"
                  label
                  color="light-blue-accent-4"
                  class="font-weight-bold ml-2"
                  @click="_copyToClipboard(currentKv.key)"
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
                :session="session"
                class="kvTree"
                @on-click="showKVUnwrapped"
          ></Tree>
          <v-sheet class="loadMoreArea d-flex align-center justify-center"
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
                <span class="count  user-select-none" title="The number of keys loaded">({{ kvCount }})</span>
              </template>
            </v-btn>
            <p v-else class="count text-center text-medium-emphasis user-select-none" title="The number of keys loaded">
              Loaded {{ kvCount }} keys</p>

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
              <v-chip
                  v-if="session.keyCollectionSet!.has(currentKv.key)"
                  class="ml-2 mt-2"
                  density="compact"
                  @click="removeCollectionKey(currentKv.key)"
                  title="Remove from collections"
                  text="Remove"
              >
                <template #prepend>
                  <v-icon color="#ced10a" class="mr-2">mdi-star</v-icon>
                </template>
              </v-chip>
              <v-chip
                  v-else
                  class="ml-2 mt-2"
                  density="compact"
                  title="Add to collections"
                  @click="addCollectionKey(currentKv.key)"
                  text="Collect"
              >
                <template #prepend>
                  <v-icon color="#ced10a" class="mr-2">mdi-star-outline</v-icon>
                </template>
              </v-chip>

              <v-chip
                  v-if="session.keyMonitorMap![currentKv.key]"
                  class="ml-2 mt-2"
                  density="compact"
                  title="Edit monitor rule"
                  @click="editKeyMonitor(currentKv.key)"
                  text="Edit"
              >
                <template #prepend>
                  <v-icon color="#cc8f53" class="mr-2">mdi-robot</v-icon>
                </template>
              </v-chip>
              <v-chip
                  v-else
                  class="ml-2 mt-2"
                  density="compact"
                  title="Add to monitor list"
                  @click="addKeyMonitor(currentKv.key)"
                  text="Add"
              >
                <template #prepend>
                  <v-icon color="#cc8f53" class="mr-2">mdi-robot-outline</v-icon>
                </template>
              </v-chip>

              <v-spacer></v-spacer>

              <v-tooltip location="top"
                         :text="_isMac() ? '⌘ + S' : 'Ctrl + S'"
              >
                <template v-slot:activator="{ props }">
                  <v-btn
                      v-bind="props"
                      :disabled="!currentKvChanged"
                      color="primary"
                      size="small"
                      @click="saveKV"
                      text="Save"
                      class="mr-2 text-none"
                      :loading="loadingStore.save"
                      prepend-icon="mdi-content-save-outline"
                  ></v-btn>
                </template>
              </v-tooltip>

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
              <div class="editor-alert" v-if="editorAlert.enable">
                <v-alert v-if="editorAlert.type === 'kubernetes'" 
                        density="compact"
                        :rounded="false"
                        class="pa-1 text-medium-emphasis editor-alert-item"
                        :style="`display: ${editorAlert.show ? 'block' : 'none'};`"
                        >
                  <v-layout>
                    <p>The kubernetes storage format is protobuf and is automatically formatted into a <strong>readonly</strong> json format.</p>
                    <span class="editor-alert-link pl-2" @click="showFormattedValue = !showFormattedValue">Recover</span>
                    <v-spacer></v-spacer>
                    
                    <v-icon @click="editorAlert.show = false" class="mr-2">mdi-chevron-double-up</v-icon>
                  </v-layout>
                </v-alert>
                <v-icon class="editor-alert-expend-link text-medium-emphasis"
                        v-show="!editorAlert.show"
                        @click="editorAlert.show = true"
                >mdi-chevron-double-down</v-icon>
              </div>
              <editor ref="editorRef"
                      :key="currentKv.key"
                      :value="editorContent"
                      :config="editorConfig"
                      @change="editorChange"
                      @change-language="editorChangeLanguage"
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
                  <span class="editor-footer-item cursor-pointer"
                        @click="_copyToClipboard(currentKv.createRevision)"><strong>Create Revision</strong>: {{
                      currentKv.createRevision
                    }}</span>
                  <span class="editor-footer-item cursor-pointer"
                        @click="_copyToClipboard(currentKv.modRevision)"><strong>Modify Revision</strong>: {{
                      currentKv.modRevision
                    }}</span>
                  <span class="editor-footer-item cursor-pointer"
                        @click="_copyToClipboard(currentKv.lease)"
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
        max-width="1200px"
        scrollable
    >
      <v-card
          :min-width="500"
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

          <v-alert v-if="versionDiffInfo.useFormattedValue"
                   icon="mdi-check-circle-outline"
                   density="compact"
          >
          It has automatically used the formatted content.
          </v-alert>
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
                          prepend-inner-icon="mdi-file-document"
                          :prefix="session.namespace"
                          hide-details
                          readonly
            ></v-text-field>
          </v-layout>
          <v-layout class="mb-5">
            <span class="new-key-form-label">Key: </span>
            <v-text-field v-model="newKeyDialog.key"
                          density="comfortable"
                          prepend-inner-icon="mdi-file-document"
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

    <!--   key收藏弹窗-->
    <v-dialog
        v-model="collectionDialog"
        eager
        transition="slide-x-reverse-transition"
        scrollable
        class="collection-drawer"
        contained
    >

      <v-card
          :rounded="false"
          title="My Collections"
      >
        <template #prepend>
          <v-icon color="#ced10a">mdi-star</v-icon>
        </template>
        <v-card-item style="height: calc(100% - 64px);">
          <v-text-field
              v-model="addCollectionKeyForm"
              type="text"
              append-inner-icon="mdi-plus"
              density="compact"
              variant="solo-filled"
              hide-details
              single-line
              clearable
              placeholder="Enter key to add to collections"
              @click:append-inner="addCollectionKey(addCollectionKeyForm); addCollectionKeyForm = '';"
          ></v-text-field>
          <div style="height: calc(100% - 40px);overflow: auto;width: 100%;">
            <Tree ref="kvCollectionTree"
                  :tree-id="`kv-collection-tree-${new Date().getTime()}`"
                  :key-splitter="KEY_SPLITTER"
                  :session="session"
                  :show-node-suffix="false"
                  :show-check-box="false"
                  show-hover-remove
                  :enable-select="false"
                  class="mt-2"
                  :init-items="session.keyCollection"
                  @on-click="onClickKeyCollectionTreeItem"
                  @on-click-remove="removeCollectionKey"
            ></Tree>
          </div>
        </v-card-item>
      </v-card>
    </v-dialog>

    <v-dialog
        v-model="searchDialog.show"
        max-width="800px"
        scrollable
    >
      <v-card>
        <v-card-title class="pa-0">
          <v-text-field v-model="searchDialog.inputValue"
                  :prefix="session.namespace"
                  autofocus
                  type="text"
                  @input="searchFromServer"
                  placeholder="Enter a prefix to search from the server"
                  prepend-inner-icon="mdi-magnify"
                  hide-details
          >
          <template #append-inner>
            <v-progress-circular
                v-if="searchDialog.loading"
                color="primary"
                indeterminate="disable-shrink"
                size="20"
                width="3"
            ></v-progress-circular>
          </template>
        </v-text-field>
        </v-card-title>
        <v-card-text class="pa-0">
          <v-list lines="two" v-if="searchDialog.searchResult && searchDialog.searchResult.results.length > 0">
            <v-list-item v-for="kv in searchDialog.searchResult.results"
                         :key="kv.key"
                         :title="kv.key"
                         append-icon="mdi-chevron-right"
                         @click="selectSearchItem(kv)"
            >
              <template #title>
                <span class="font-weight-bold">{{ kv.key }}</span>
              </template>
              <template #subtitle>
                Version: <i>{{ kv.version }}</i>,
                Create Revision: <i>{{ kv.createRevision }}</i>,
                Modify Revision: <i>{{ kv.modRevision }}</i>
              </template>
            </v-list-item>
          </v-list>
        </v-card-text>
        <v-card-actions class="text-medium-emphasis">
          <v-spacer/>
          <span v-if="searchDialog.searchResult">Searched {{ searchDialog.searchResult.results.length }} / {{ searchDialog.searchResult.count }}</span>
          <span v-else>Search all keys from etcd server, and display up to 50 results.</span>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>

<style scoped lang="scss">
$--action-area-height: 60px;
$--action-area-margin-bottom: 10px;
$--load-more-area-height: 32px;

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
    position: relative;
    height: calc(100% - $--editor-header-height);

    .editor-alert {
      position: absolute;
      height: max-content;
      width: 100%;
      top: 0;
      left: 0;
      z-index: 100;
      .editor-alert-item {
        font-size: 0.9em;
        transition: all ease 0.8s;
        .editor-alert-link {
          color: #9d9cf3;
          cursor: pointer;
          margin-right: 5px;
        }

        .editor-alert-link:hover {
          text-decoration: underline;
        }
      }

      .editor-alert-expend-link {
        position: absolute;
        right: 11px;
      }
    }
  }

}

.new-key-form-label {
  display: inline-block;
  width: 80px;
  line-height: 48px;
}

.kvTree {
  height: calc(100% - $--load-more-area-height);
}

.loadMoreArea {

  height: $--load-more-area-height;

  .count {
    font-size: 0.8em;
    cursor: default;
    height: $--load-more-area-height;
    line-height: $--load-more-area-height;
  }
}
</style>

<style>
.collection-drawer .v-overlay__content {
  width: 600px;
  height: 100%;
  margin: 0;
  position: absolute;
  right: 0;
  padding: 0;
  border-radius: 0;
}

.collection-drawer .v-card-item__content {
  height: 100%;
  align-self: start;
}
</style>
