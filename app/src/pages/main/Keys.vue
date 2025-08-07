<script setup lang="ts">

import {MergeView} from "@codemirror/merge";
import {EditorState, Extension} from "@codemirror/state";
import {basicSetup, EditorView} from "codemirror";
import {CodeDiff} from "v-code-diff";
import {computed, nextTick, onMounted, onUnmounted, PropType, reactive, ref, watch} from "vue";
import {useLocale, useTheme} from "vuetify";
import {VTextField} from "vuetify/components";
import {
  _alertError,
  _confirm,
  _confirmSystem,
  _copyToClipboard,
  _emitLocal,
  _listenLocal,
  _loading,
  _tipError,
  _tipInfo,
  _tipSuccess,
  _tipWarn,
  _unListenLocal,
  EventName, KVRenameDirEvent
} from "~/common/events.ts";
import {
  _deleteKV,
  _getAllKeys,
  _getAllKeysPaging,
  _getKV,
  _getKVByVersion,
  _getKVHistoryVersions,
  _handleError, _kvRenameDir,
  _kvSearchNextDir,
  _putKV,
  _putKVWithLease,
  _searchByPrefix,
  _updateKeyCollection
} from "~/common/services.ts";
import {_saveGlobalStore, _useGlobalStore, _useSettings} from "~/common/store.ts";
import {ErrorPayload, KeyMonitorConfig, SessionData} from "~/common/transport/connection.ts";
import {KeyExtendInfo, PutStrategy, SearchResult} from "~/common/transport/kv";
import {KeyValue} from "~/common/transport/kv.ts";
import {EditorConfig, EditorHighlightLanguage} from "~/common/types.ts";
import {_debounce, arraysEqual} from "~/common/utils";
import {
  _decodeBytesToString,
  _encodeStringToBytes,
  _isEmpty,
  _tryParseDiffLanguage,
  _tryParseEditorLanguage
} from "~/common/utils.ts";
import {_isMac} from "~/common/windows.ts";
import CountDownTimer from "~/components/CountDownTimer.vue";
import DragBox from "~/components/drag-area/DragBox.vue";
import DragItem from "~/components/drag-area/DragItem.vue";
import Editor from "~/components/editor/Editor.vue";
import {getLanguage} from "~/components/editor/languages.ts";
import {getTheme} from "~/components/editor/themes.ts";
import Tree, {ContextmenuKeyword} from "~/components/tree/Tree.vue";
import {Handler} from "mitt";
import CompleteInput from "~/components/CompleteInput.vue";
import {appWindow} from "@tauri-apps/api/window";
import {_deepSearchTreeNodes, TreeNode} from "~/components/tree/types.ts";

const {t} = useLocale()
const theme = useTheme()
const settings = _useSettings()

type DiffInfo = {
  version: number,
  content: string
}

//  自动移除lease失效key的开关
const AUTO_REMOVE_EXPIRED_KEY = false

const KEY_SPLITTER = computed<string>(() => {
  return settings.value.kvPathSplitter
})

const LIMIT_PER_PAGE = computed(() => {
  return settings.value.kvLimitPerPage
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
const putMergeEditorRef = ref<InstanceType<typeof HTMLElement>>()
const putMergeEditor = ref<MergeView>()
const eventUnListens = reactive<Function[]>([])
const renameDirLogListRef = ref<InstanceType<typeof HTMLDivElement>>()

const defaultEditorConfig: EditorConfig = {
  disabled: false,
  indentWithTab: true,
  tabSize: 2,
  autofocus: false,
  height: "100%",
  fontSize: "1rem",
  language: 'text'
}

const editorConfig = reactive<EditorConfig>({
  ...defaultEditorConfig
})

const newKeyEditorConfig = reactive<EditorConfig>({
  ...defaultEditorConfig
})

const loadingStore = reactive({
  save: false,
  diff: false,
  delete: false,
  deleteBatch: false,
  confirmNewKey: false,
  loadMore: false,
  getKey: false,
  renameDir: false
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
  model: <'none' | 'ttl' | 'lease'>'none',
  deleteFromKey: false
})

const versionDiffInfo = reactive({
  show: false,
  key: '',
  keyBytes: <number[] | undefined>undefined,
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

const putMergeDialog = reactive({
  show: false,
  existValue: "",
  existVersion: 0,
  request: {
    key: "",
    value: "",
    ttl: <undefined | number>undefined
  },
  successCallback: <Function | undefined>undefined,
  failedCallback: <Function | undefined>undefined
})

const renameDirDialog = reactive({
  show: false,
  originPrefix: "",
  newPrefix: "",
  deleteOriginKeys: true,
  putStrategy: <PutStrategy> 'Cover',
  state: <'none' | 'started' | 'ended' | 'failed'> 'none',
  logs: <KVRenameDirEvent[]>[]
})

watch(
    () => theme,
    () => {
      renderMergeViewEditor()
    },
    {
      deep: true,
    }
)

watch(
    () => settings.value,
    (newVal, oldVal) => {
      if (newVal.editorDarkTheme != oldVal.editorDarkTheme || newVal.editorLightTheme != oldVal.editorLightTheme) {
        renderMergeViewEditor()
      }
    },
    {
      deep: true,
    }
)

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

onMounted(async () => {
  //  海量数据加载时会导致页面其他动画卡顿，这里延迟加载
  setTimeout(() => {
    nextTick(() => {
      _loading(true)
      refreshAllKeys().finally(() => {
        _loading(false)
      })
    })
  }, 200)

  const keyMonitorConfigChangeEventHandler: Handler<any> = e => {
    if (e.session == props.session?.id) {
      let key = e.key as string
      kvTree.value?.refreshDiyDom(key)
    }
  }
  _listenLocal(EventName.KEY_MONITOR_CONFIG_CHANGE, keyMonitorConfigChangeEventHandler)
  eventUnListens.push(() => _unListenLocal(EventName.KEY_MONITOR_CONFIG_CHANGE, keyMonitorConfigChangeEventHandler))

  putMergeEditor.value = new MergeView({
    a: {
      doc: putMergeDialog.request.value
    },
    b: {
      doc: putMergeDialog.existValue,
      extensions: [
        basicSetup,
        EditorView.editable.of(false),
        EditorState.readOnly.of(true),
      ]
    },
    parent: putMergeEditorRef.value!
  })

  eventUnListens.push(await appWindow.listen<KVRenameDirEvent>(EventName.RENAME_DIR_EVENT, e => {
    const event = e.payload
    renameDirDialog.logs.push(event)
    if (event.success && kvTree.value) {
      const key = _decodeBytesToString(event.key)
      if (event.action == 'Put') {
        kvTree.value.addItemToTree(key, true)
      } else if (event.action == 'Delete') {
        removeKeysFromTree([key])
      }
    }
    renameDirLogScrollToBottom()
  }))

  eventUnListens.push(await appWindow.listen(EventName.RENAME_DIR_START_EVENT, () => {
    renameDirDialog.state = 'started'
  }))

  eventUnListens.push(await appWindow.listen(EventName.RENAME_DIR_END_EVENT, () => {
    renameDirDialog.state = 'ended'
    loadingStore.renameDir = false
    renameDirLogScrollToBottom()
  }))

  eventUnListens.push(await appWindow.listen(EventName.RENAME_DIR_ERR_EVENT, () => {
    renameDirDialog.state = 'failed'
    loadingStore.renameDir = false
    renameDirLogScrollToBottom()
  }))
})

onUnmounted(() => {
  clearAllKeyLeaseListener()

  for (let eventUnListen of eventUnListens) {
    eventUnListen()
  }
})

const refreshAllKeys = (): Promise<any> => {
  currentKv.value = undefined
  kvCount.value = 0
  clearAllKeyLeaseListener()
  kvTree.value?.rerender()

  if (settings.value.kvPaginationQuery && !enforceLoadAllKey.value) {
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

const showNewKeyDialog = (presetKey?: string) => {
  newKeyDialog.key = presetKey || ''
  newKeyDialog.ttl = ''
  newKeyDialog.lease = ''
  newKeyDialog.value = ''
  newKeyDialog.fromKey = ''
  newKeyDialog.model = 'none'
  newKeyDialog.title = t('main.keys.addKey')
  newKeyDialog.copyAndSave = false
  newKeyDialog.show = true

  newKeyEditorConfig.language = editorConfig.language
}

const showCopyAndSaveDialog = (title: string, fromKey: string, fromValue: string, deleteFromKey: boolean) => {
  newKeyDialog.key = fromKey
  newKeyDialog.ttl = ''
  newKeyDialog.lease = ''
  newKeyDialog.model = 'none'
  newKeyDialog.fromKey = fromKey
  newKeyDialog.value = fromValue
  newKeyDialog.title = title
  newKeyDialog.copyAndSave = true
  newKeyDialog.deleteFromKey = deleteFromKey
  newKeyDialog.show = true

  newKeyEditorConfig.language = _tryParseEditorLanguage(fromKey, fromValue, undefined, props.session?.namespace)
}

const putKey = () => {
  if (_isEmpty(newKeyDialog.key)) {
    _tipWarn(t('main.keys.emptyKeyTip'))
    return
  }
  if (newKeyDialog.model === 'ttl' && _isEmpty(newKeyDialog.ttl)) {
    _tipWarn(t('main.keys.invalidTtlTip'))
    return
  }
  if (newKeyDialog.model === 'lease' && _isEmpty(newKeyDialog.lease)) {
    _tipWarn(t('main.keys.invalidLeaseTip'))
    return
  }
  if (newKeyDialog.copyAndSave && newKeyDialog.key === newKeyDialog.fromKey) {
    _tipWarn(t('main.keys.keyNotChangedTip'))
    return;
  }
  let key = newKeyDialog.key
  let value: number[] = newKeyEditorRef.value!.readDataBytes()
  let promise: Promise<void>
  if (newKeyDialog.model === 'lease') {
    promise = _putKVWithLease(props.session?.id, key, value, newKeyDialog.lease)
  } else {
    let ttl = newKeyDialog.model === 'none' ? undefined : parseInt(newKeyDialog.ttl)
    promise = new Promise<void>((resolve, reject) => {
      _putKV(props.session?.id, key, value, 0, ttl).then((result) => {
        if (result.success) {
          resolve()
        } else {
          putMergeDialog.request.key = key;
          putMergeDialog.request.value = newKeyEditorRef.value!.readDataString()

          putMergeDialog.request.ttl = ttl
          putMergeDialog.existValue = _decodeBytesToString(result.existValue!)
          putMergeDialog.existVersion = result.existVersion!
          putMergeDialog.successCallback = resolve
          putMergeDialog.failedCallback = reject

          putMergeDialog.show = true
          renderMergeViewEditor()
        }
      }).catch(e => {
        reject(e)
      })
    })
  }

  loadingStore.confirmNewKey = true
  promise.then(() => {
    _tipSuccess(t('common.successTip'))
    newKeyDialog.show = false

    //  重命名：删除源key
    if (newKeyDialog.copyAndSave && newKeyDialog.deleteFromKey) {
      const fromKey = newKeyDialog.fromKey
      _deleteKV(props.session?.id, [fromKey], []).then(() => {
        if (currentKv.value && currentKv.value.key == fromKey) {
          currentKv.value = undefined
        }
        removeKeysFromTree([fromKey])
      }).catch(e => {
        _handleError({
          e,
          session: props.session
        })
      })
    }

    kvTree.value?.addItemToTree(key, true)
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
    kvTree.value?.addItemToTree(
        kv.key,
        ignoreIfExist,
        {
          keyBytes: kv.keyBytes,
          keyEncodedUtf8: kv.keyEncodedUtf8,
        }
    )
  }
}

const deleteKeyBatch = (nodes: TreeNode[]) => {
  if (nodes.length == 0) {
    _tipInfo(t('main.keys.deleteEmptyKeyTip'))
    return
  }

  const keysEncodedUtf8: string[] = []
  const keys: string[] = []
  const keyBytes: number[][] = []
  let containsCurrentKV = false
  for (const node of nodes) {
    if (node.isParent) {
      continue
    }
    keysEncodedUtf8.push(node.id)
    if (node.keyInfo) {
      if (node.keyInfo.keyEncodedUtf8) {
        if (!containsCurrentKV && currentKv.value && currentKv.value.key == node.id) {
          containsCurrentKV = true
        }
        keys.push(node.id)
      } else {
        if (!containsCurrentKV && currentKv.value && arraysEqual(currentKv.value.keyBytes, node.keyInfo.keyBytes)) {
          containsCurrentKV = true
        }
        keyBytes.push(node.keyInfo.keyBytes)
      }
    } else {
      if (!containsCurrentKV && currentKv.value && currentKv.value.key == node.id) {
        containsCurrentKV = true
      }
      keys.push(node.id)
    }
  }

  let message = t('main.keys.deleteConfirm') + "<br/><br/><strong>"

  const showCount = 20
  if (keysEncodedUtf8.length >= showCount) {
    message += keysEncodedUtf8.slice(0, showCount).join('<br/>')
    message += `<br/><br/> ... ${t('common.omit')} ${keysEncodedUtf8.length - showCount} keys`
  } else {
    message += keysEncodedUtf8.join('<br/>')
  }

  message += '</strong>'
  _confirmSystem(message).then(() => {
    _loading(true, t('main.keys.deletingKeyTip'))
    loadingStore.deleteBatch = true
    _deleteKV(props.session?.id, keys, keyBytes).then(() => {
      if (containsCurrentKV) {
        currentKv.value = undefined
      }
      removeKeysFromTree(keysEncodedUtf8)
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

const showKV = (key: string, keyInfo?: KeyExtendInfo): Promise<void> => {
  return new Promise((resolve, reject) => {
    loadingStore.getKey = true
    _getKV(props.session?.id, key, keyInfo && !keyInfo.keyEncodedUtf8 ? keyInfo.keyBytes : undefined).then((kv) => {
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

const showKVUnwrapped = (key: string, keyInfo?: KeyExtendInfo) => {
  showKV(key, keyInfo).then(() => {
  }).catch(e => {
    console.error(e)
  })
}

const addCollectionKey = (key: string, keyEncodedUtf8?: boolean) => {
  if (_isEmpty(key)) {
    return
  }
  if (!keyEncodedUtf8) {
    _alertError(t('main.keys.addNonUtf8ToCollectionTip'))
    return;
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
      _putKV(props.session?.id, kv!.key, value, kv!.version).then((result) => {
        if (result.success) {
          currentKvChanged.value = false
          result.finalKv!.value = value
          currentKv.value = result.finalKv
        } else {
          putMergeDialog.request.key = kv!.key
          putMergeDialog.request.value = editorRef.value!.readDataString()
          putMergeDialog.request.ttl = undefined
          putMergeDialog.existValue = _decodeBytesToString(result.existValue!)
          putMergeDialog.existVersion = result.existVersion!
          putMergeDialog.successCallback = (finalKv: KeyValue, value: number[]) => {
            currentKvChanged.value = false
            finalKv.value = value
            currentKv.value = finalKv
          }
          putMergeDialog.failedCallback = undefined
          putMergeDialog.show = true
          renderMergeViewEditor()
        }
      }).catch(e => {
        _handleError({
          e,
          session: props.session
        })
      }).finally(() => {
        loadingStore.save = false
      })
    }

    if (settings.value.kvCheckFormatBeforeSave) {
      editorRef.value.tryFormatContent().then(() => {
        doSave()
      }).catch(() => {
        _confirm(t('common.warning'), t('main.keys.formatCheckConfirm')).then(() => {
          doSave()
        }).catch(() => {
        })
      })
    } else {
      doSave()
    }
  }
}

const loadVersionDiff = (key: string, info?: KeyExtendInfo) => {
  if (!key) {
    return
  }

  loadingStore.diff = true
  versionDiffInfo.key = key

  if (info && !info.keyEncodedUtf8) {
    versionDiffInfo.keyBytes = info.keyBytes
  } else {
    versionDiffInfo.keyBytes = undefined
  }
  _getKV(props.session?.id, key, versionDiffInfo.keyBytes).then(dataB => {
    versionDiffInfo.version = dataB.version
    versionDiffInfo.createRevision = dataB.createRevision
    versionDiffInfo.modRevision = dataB.modRevision

    //  当前版本
    versionDiffInfo.B.version = versionDiffInfo.modRevision
    if (dataB!.formattedValue) {
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
        dataB.modRevision,
        versionDiffInfo.keyBytes,
    ).then(versions => {
      if (versions.length < 2) {
        _tipWarn(t('main.keys.noMultipleVersionTip'))
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
  _getKVByVersion(props.session?.id, versionDiffInfo.key, info.version, versionDiffInfo.keyBytes).then(data => {
    info.content = data.formattedValue ? data.formattedValue.value : _decodeBytesToString(data.value)

    if (!versionDiffInfo.show) {
      versionDiffInfo.show = true
    }
  }).catch(e => {
    _handleError({
      e,
      prefix: t('main.keys.failedToLoadReversion', {version: info.version}),
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
    item.subtitle = t('main.keys.versionTagCreate')
    item['append-icon'] = 'mdi-creation-outline'
  }
  if (version == versionDiffInfo.modRevision) {
    item.subtitle = t('main.keys.versionTagLatest')
    item['append-icon'] = 'mdi-new-box'
  }
  return item
}

const deleteKey = (key: string, info?: KeyExtendInfo) => {
  _confirmSystem(`${t('main.keys.deleteKeyConfirm')} <strong>${key}</strong>`).then(() => {
    loadingStore.delete = true
    const utf8EncodedKeys: string[] = []
    const unUtf8EncodedKeys: number[][] = []

    if (info && !info.keyEncodedUtf8) {
      unUtf8EncodedKeys.push(info.keyBytes)
    } else {
      utf8EncodedKeys.push(key)
    }

    _deleteKV(
        props.session?.id,
        utf8EncodedKeys,
        unUtf8EncodedKeys
    ).then(() => {
      currentKv.value = undefined
      removeKeysFromTree([key])
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

const onClickKeyCollectionTreeItem = (key: string, keyInfo?: KeyExtendInfo) => {
  kvTree.value?.addItemToTree(key, true, keyInfo)
  kvTree.value?.selectItem(key)
  showKV(key, keyInfo).then(() => {
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

const addKeyMonitor = (key: string, isPrefix: boolean, keyEncodedUtf8?: boolean) => {
  if (!keyEncodedUtf8) {
    _alertError(t('main.keys.addNonUtf8ToMonitorTip'))
    return
  }
  _emitLocal(EventName.EDIT_KEY_MONITOR, {
    session: props.session?.id,
    edit: false,
    key,
    isPrefix
  })
}

const openSearchDialog = () => {
  searchDialog.inputValue = ''
  searchDialog.searchResult = null
  searchDialog.show = true
  searchDialog.loading = false
}

const selectSearchItem = (kv: KeyValue) => {
  showKV(kv.key, {
    keyBytes: kv.keyBytes,
    keyEncodedUtf8: kv.keyEncodedUtf8,
  })
  kvTree.value?.selectItem(kv.key)
  searchDialog.show = false
}

const searchFromServer = _debounce(() => {
  if (_isEmpty(searchDialog.inputValue)) {
    searchDialog.searchResult = null
    return
  }
  searchDialog.loading = true
  _searchByPrefix(props.session?.id, searchDialog.inputValue).then((data: SearchResult) => {
    searchDialog.searchResult = data

    if (data) {
      addDataListToTree(data.results, true)
    }
  }).finally(() => {
    searchDialog.loading = false
  })
}, 1000)

const renderMergeViewEditor = () => {
  if (!putMergeDialog.show) {
    return
  }
  const language = _tryParseEditorLanguage(putMergeDialog.request.key, putMergeDialog.request.value)

  const extensions: Extension[] = []

  const languageExtension = getLanguage(language)
  if (languageExtension) {
    extensions.push(languageExtension)
  }

  extensions.push(getTheme(theme.global.name.value))
  nextTick(() => {
    if (putMergeEditor.value) {
      putMergeEditor.value.destroy()
    }
    putMergeEditor.value = new MergeView({
      a: {
        doc: putMergeDialog.request.value,
        extensions: [
          ...extensions,
          basicSetup
        ],
      },
      b: {
        doc: putMergeDialog.existValue,
        extensions: [
          ...extensions,
          basicSetup,
          EditorView.editable.of(false),
          EditorState.readOnly.of(true)
        ]
      },
      gutter: true,
      revertControls: 'b-to-a',
      parent: putMergeEditorRef.value!,
      highlightChanges: true,
    })
  })
}

const cancelMergeDialog = () => {
  putMergeDialog.show = false
  if (putMergeDialog.failedCallback) {
    putMergeDialog.failedCallback()
  }
}

const confirmMergeDialog = () => {
  putMergeDialog.show = false
  const key = putMergeDialog.request.key
  const value = putMergeEditor.value!.a!.state.doc.toString()
  const ttl = putMergeDialog.request.ttl
  const valueBytes = _encodeStringToBytes(value)
  _putKV(props.session?.id, key, valueBytes, putMergeDialog.existVersion, ttl).then((result) => {
    if (result.success) {
      if (putMergeDialog.successCallback) {
        putMergeDialog.successCallback(result.finalKv, valueBytes)
      }
    } else {
      putMergeDialog.request.key = key
      putMergeDialog.request.value = value
      putMergeDialog.request.ttl = ttl
      putMergeDialog.existValue = _decodeBytesToString(result.existValue!)
      putMergeDialog.existVersion = result.existVersion!

      putMergeDialog.show = true
      renderMergeViewEditor()
    }
  }).catch(e => {
    if (putMergeDialog.failedCallback) {
      putMergeDialog.failedCallback(e)
    }
  })
}

const searchNextDir = (value: string | null): Promise<string[]> => {
  return searchNext(value, false)
}

const searchNextNode = (value: string | null): Promise<string[]> => {
  return searchNext(value, true)
}

const searchNext = (value: string | null, includeFile: boolean): Promise<string[]> => {
  const prefix = value || ""
  return _kvSearchNextDir(props.session?.id, prefix, includeFile).catch(e => {
    _handleError({
      e,
      session: props.session
    })
    return []
  })
}

const putAnyway = (key: string, value: string, version: number) => {
  _confirmSystem(t('main.keys.putAnywayConfirm',{ version })).then(() => {
    _loading(true)
    _putKV(props.session?.id, key, _encodeStringToBytes(value), -1).then((result) => {
      if (result.success) {
        versionDiffInfo.show = false
        showKV(key)
      } else {
        _tipError(t('common.failed'))
      }
    }).catch((e) => {
      _handleError({
        e,
        session: props.session
      })
    }).finally(() => {
      _loading(false)
    })
  }).catch(() => {
  })
}

const onClickContextmenu = (keyword: ContextmenuKeyword, node: TreeNode) => {
  if (node) {
    const key = node.id
    if (keyword == 'addToMonitor') {
      addKeyMonitor(key, node.isParent, true)
      return
    } else if (keyword == 'editMonitor') {
      editKeyMonitor(key)
      return
    }

    if (node.isParent) {
      //  修改目录名字
      if (keyword == 'rename') {
        renameDirDialog.originPrefix = key
        renameDirDialog.newPrefix = key
        renameDirDialog.deleteOriginKeys = true
        renameDirDialog.putStrategy = 'Cover'
        renameDirDialog.state = 'none'
        renameDirDialog.logs = []
        renameDirDialog.show = true
      }
      else if (keyword == 'addKey') {
        showNewKeyDialog(key)
      } else if (keyword == 'delete') {
        const nodes:TreeNode[] = [node]
        _deepSearchTreeNodes(node, nodes)
        deleteKeyBatch(nodes)
      }
    } else {
      //  修改key名字
      if (keyword == 'rename' || keyword == 'copyAndSave') {
        _loading(true)
        _getKV(props.session?.id, key).then((kv) => {
          showCopyAndSaveDialog(
              keyword == 'rename' ? t('main.keys.rename') : t('main.keys.copyAndSave'),
              key,
              _decodeBytesToString(kv.value),
              keyword == 'rename'
          )
        }).catch(e => {
          if (e.errType && e.errType == 'ResourceNotExist') {
            removeKeysFromTree([key])
          }
          _handleError({
            e,
            session: props.session
          })
        }).finally(() => {
          _loading(false)
        })
      } else if (keyword == 'delete') {
        deleteKey(key)
      } else if (keyword == 'addToCollection') {
        addCollectionKey(key, true)
      } else if (keyword == 'removeFromCollection') {
        removeCollectionKey(key)
      } else if (keyword == 'versionDiff') {
        loadVersionDiff(key)
      }
    }
  }
}

const renameDir = () => {
  if(renameDirDialog.originPrefix === renameDirDialog.newPrefix) {
    _tipWarn(t('main.keys.pathNameNotChanged'))
    return
  }
  loadingStore.renameDir = true
  _kvRenameDir(
      props.session?.id,
      renameDirDialog.originPrefix,
      renameDirDialog.newPrefix,
      renameDirDialog.deleteOriginKeys,
      renameDirDialog.putStrategy
  ).then(() => {
  }).catch(e => {
    if (e.errType && e.errType == 'LimitedError') {
      const count = (e as ErrorPayload).data!.count
      _alertError(t('main.keys.renameFailed', { count }))
    } else {
      _handleError({
        e,
        session: props.session
      })
    }
    loadingStore.renameDir = false
  })
}

const renameDirLogScrollToBottom = () => {
  nextTick(() => {
    if (renameDirLogListRef.value) {
      renameDirLogListRef.value.scrollTop = renameDirLogListRef.value.scrollHeight
    }
  })
}
</script>

<template>
  <div class="fill-height overflow-y-auto">
    <v-layout class="action-area pa-5">
      <v-btn v-bind="props"
             variant="tonal"
             size="small"
             icon="mdi-refresh"
             @click="refreshAllKeys"
             :loading="loadingStore.loadMore"
             :title="t('common.refresh')"
      />

      <v-btn class="text-none ml-2"
             prepend-icon="mdi-file-document-plus-outline"
             color="green"
             @click="showNewKeyDialog()"
             :text="t('main.keys.addKey')"
      />
      <v-btn class="text-none ml-2"
             prepend-icon="mdi-file-document-minus-outline"
             color="red"
             @click="deleteKeyBatch(kvTree!.getSelectedItems())"
             :loading="loadingStore.deleteBatch"
             :text="t('main.keys.deleteKeys')"
      />

      <v-btn class="text-none ml-2"
             prepend-icon="mdi-star"
             color="yellow"
             @click="collectionDialog = true"
             :text="t('main.keys.myCollections')"
      />

      <v-btn class="text-none ml-2"
             v-bind="props"
             prepend-icon="mdi-text-box-search-outline"
             color="blue-lighten-1"
             @click="openSearchDialog"
             :text="t('main.keys.search')"
             :title="t('main.keys.searchBtnTitle')"
      />

      <v-spacer></v-spacer>

      <v-tooltip
          v-if="session.namespace"
          location="top"
          :text="t('main.home.connector.namespace')"
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
      <v-tooltip v-if="currentKv" location="top" :text="t('main.keys.currentKey')">
        <template v-slot:activator="{ props }">
          <v-chip v-bind="props" label
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
                enable-contextmenu
                class="kvTree"
                @on-click="showKVUnwrapped"
                @click:contextmenu="onClickContextmenu"
          />
          <v-sheet class="loadMoreArea d-flex align-center justify-center">
            <v-btn v-if="paginationKeyCursor != undefined"
                   block
                   density="compact"
                   color="cyan-darken-4"
                   class="text-none border-none user-select-none"
                   style="border-radius: 0;"
                   :text="t('main.keys.loadMore')"
                   @click="loadNextPage"
                   prepend-icon="mdi-book-open-page-variant-outline"
            >
              <template #append>
                <span
                    class="count user-select-none"
                    :title="t('main.keys.loadedLineTitle')"
                >({{ kvCount }})</span>
              </template>
            </v-btn>
            <p
                v-else
                class="count text-center text-medium-emphasis user-select-none"
                :title="t('main.keys.loadedLineTitle')"
            >{{ t('main.keys.loadedLine', {count: kvCount}) }}</p>

          </v-sheet>
        </drag-item>
        <drag-item ref="kvEditorContainerRef" style="width: calc(100% - 300px)" :show-resize-line="false">
          <v-overlay v-model="loadingStore.getKey"
                     persistent
                     contained
                     class="align-center justify-center ma-0"
                     :z-index="100"
          >
            <v-progress-circular
                color="primary"
                size="40"
                indeterminate
            />
          </v-overlay>

          <div v-if="currentKv" class="fill-height">
            <v-layout class="editor-header">
              <v-chip v-if="session.keyCollectionSet!.has(currentKv.key)"
                      class="ml-2 mt-2"
                      density="compact"
                      @click="removeCollectionKey(currentKv.key)"
                      :title="t('main.keys.removeCollectionTitle')"
                      :text="t('common.remove')"
              >
                <template #prepend>
                  <v-icon color="#ced10a" class="mr-2">mdi-star</v-icon>
                </template>
              </v-chip>
              <v-chip v-else
                      class="ml-2 mt-2"
                      density="compact"
                      :title="t('main.keys.collectBtnTitle')"
                      @click="addCollectionKey(currentKv.key, currentKv.keyEncodedUtf8)"
                      :text="t('main.keys.collect')"
              >
                <template #prepend>
                  <v-icon color="#ced10a" class="mr-2">mdi-star-outline</v-icon>
                </template>
              </v-chip>

              <v-chip v-if="session.keyMonitorMap![currentKv.key]"
                      class="ml-2 mt-2"
                      density="compact"
                      :title="t('main.keys.editBtnTitle')"
                      @click="editKeyMonitor(currentKv.key)"
                      :text="t('common.edit')"
              >
                <template #prepend>
                  <v-icon color="#cc8f53" class="mr-2">mdi-robot</v-icon>
                </template>
              </v-chip>
              <v-chip v-else
                      class="ml-2 mt-2"
                      density="compact"
                      :title="t('main.keys.addMonitorBtnTitle')"
                      @click="addKeyMonitor(currentKv.key, false, currentKv.keyEncodedUtf8)"
                      :text="t('common.add')"
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
                  <v-btn v-bind="props"
                         :disabled="!currentKvChanged"
                         color="primary"
                         size="small"
                         @click="saveKV"
                         :text="t('common.save')"
                         class="mr-2 text-none"
                         :loading="loadingStore.save"
                         prepend-icon="mdi-content-save-outline"
                  />
                </template>
              </v-tooltip>

              <v-btn color="cyan-darken-1"
                     size="small"
                     @click="loadVersionDiff(currentKv!.key, currentKv)"
                     :text="t('main.keys.versionDiff')"
                     class="mr-2 text-none"
                     :loading="loadingStore.diff"
                     prepend-icon="mdi-vector-difference"
              />
              <v-btn color="light-green-darken-1"
                     size="small"
                     :text="t('main.keys.copyAndSave')"
                     class="mr-2 text-none"
                     prepend-icon="mdi-content-copy"
                     @click="showCopyAndSaveDialog(t('main.keys.copyAndSave'), currentKv.key, _decodeBytesToString(currentKv.value), false)"
              />
              <v-btn color="deep-orange-darken-1"
                     size="small"
                     @click="deleteKey(currentKv.key, currentKv)"
                     :loading="loadingStore.delete"
                     :disabled="!currentKv"
                     :text="t('common.delete')"
                     class="mr-2 text-none"
                     prepend-icon="mdi-trash-can-outline"
              />
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
                    <p v-html="t('main.keys.k8sFormatNotice')"></p>
                    <span class="editor-alert-link pl-2"
                          @click="showFormattedValue = !showFormattedValue">{{t('common.recover')}}</span>
                    <v-spacer></v-spacer>

                    <v-icon @click="editorAlert.show = false" class="mr-2">mdi-chevron-double-up</v-icon>
                  </v-layout>
                </v-alert>
                <v-icon class="editor-alert-expend-link text-medium-emphasis"
                        v-show="!editorAlert.show"
                        @click="editorAlert.show = true"
                >mdi-chevron-double-down
                </v-icon>
              </div>
              <editor ref="editorRef"
                      :key="currentKv.key"
                      :value="editorContent"
                      :config="editorConfig"
                      @change="editorChange"
                      @change-language="editorChangeLanguage"
                      @save="editorSave"
              >
                <template #footer>
                  <span class="editor-footer-item ml-0" v-if="currentKv.leaseInfo">
                    <v-tooltip location="top" :text="`${t('main.keys.grantedTtl')}: ${currentKv.leaseInfo.grantedTtl} s`">
                      <template v-slot:activator="{ props }">
                        <span class="text-secondary user-select-none" v-bind="props">
                          <CountDownTimer :value="currentKv.leaseInfo.ttl"/>
                        </span>
                      </template>
                    </v-tooltip>
                  </span>
                  <v-spacer></v-spacer>
                  <span class="editor-footer-item"><strong>{{t('common.version')}}}</strong>: {{ currentKv.version }}</span>
                  <span class="editor-footer-item cursor-pointer"
                        @click="_copyToClipboard(currentKv.createRevision)"><strong>{{t('main.keys.createRevision')}}</strong>: {{
                      currentKv.createRevision
                    }}</span>
                  <span class="editor-footer-item cursor-pointer"
                        @click="_copyToClipboard(currentKv.modRevision)">
                    <strong>{{t('main.keys.modifyRevision')}}</strong>: {{currentKv.modRevision }}</span>
                  <span class="editor-footer-item cursor-pointer"
                        @click="_copyToClipboard(currentKv.lease)"
                        v-if="currentKv.lease != '0'">
                    <strong>{{t('common.lease')}}</strong>: {{ currentKv.lease }}</span>
                </template>
              </editor>
            </div>
          </div>

          <div v-else class="no-key-preview fill-height">
            <v-empty-state icon="mdi-text-box-edit-outline"
                           :headline="t('main.keys.emptyStateHeadline')"
                           :title="t('main.keys.emptyStateTitle')"
                           class="mx-auto my-auto user-select-none">
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
        style="z-index:1200;"
    >
      <v-card :min-width="500" :title="versionDiffInfo.key" :key="versionDiffInfo.key">
        <template v-slot:prepend>
          <v-icon>mdi-vector-difference</v-icon>
        </template>
        <template v-slot:append>
          <v-icon class="cursor-pointer" @click="versionDiffInfo.show = false">mdi-close</v-icon>
        </template>
        <v-card-text>

          <v-alert
              v-if="versionDiffInfo.useFormattedValue"
              icon="mdi-check-circle-outline"
              density="compact"
          >
            {{t('main.keys.diffDialogAlert')}}
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
                :label="t('main.keys.versionA')"
                @update:model-value="loadDiff(versionDiffInfo.A)"
            />
            <v-btn class="text-none ml-2"
                   prepend-icon="mdi-gesture-swipe-up"
                   color="primary"
                   @click="putAnyway(versionDiffInfo.key, versionDiffInfo.A.content, versionDiffInfo.A.version)"
                   :text="t('main.keys.putThisVersion')"
                   :density="null"
                   :disabled="versionDiffInfo.A.version == versionDiffInfo.modRevision"
            />
            <v-spacer></v-spacer>
            <v-btn class="text-none mr-2"
                   prepend-icon="mdi-gesture-swipe-up"
                   color="primary"
                   @click="putAnyway(versionDiffInfo.key, versionDiffInfo.B.content, versionDiffInfo.B.version)"
                   :text="t('main.keys.putThisVersion')"
                   :density="null"
                   :disabled="versionDiffInfo.B.version == versionDiffInfo.modRevision"
            />
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
                :label="t('main.keys.versionB')"
                @update:model-value="loadDiff(versionDiffInfo.B)"
            />
          </v-layout>

          <code-diff
              style="max-height: 60vh;min-height: 40vh;"
              :old-string="versionDiffInfo.A.content"
              :filename="`${t('common.revision')}: ${versionDiffInfo.A.version}`"
              :new-string="versionDiffInfo.B.content"
              :new-filename="`${t('common.revision')}: ${versionDiffInfo.B.version}`"
              :theme="isDarkTheme ? 'dark' : 'light'"
              :language="versionDiffInfo.language"
              output-format="side-by-side"
          />
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
            <span class="inline-label input-label">{{ t('main.keys.fromLabel') }}: </span>
            <v-text-field
                v-model="newKeyDialog.fromKey"
                density="comfortable"
                prepend-inner-icon="mdi-file-document"
                :prefix="session.namespace"
                hide-details
                readonly
            />
          </v-layout>
          <v-layout class="mb-5" v-show="newKeyDialog.copyAndSave">
            <span class="inline-label checkbox-label"></span>
            <v-checkbox
                :label="t('main.keys.deleteFrom')"
                v-model="newKeyDialog.deleteFromKey"
                hide-details
            />
          </v-layout>
          <v-layout class="mb-5 overflow-visible">
            <span class="inline-label input-label" v-if="newKeyDialog.copyAndSave">{{ t('main.keys.toLabel') }}: </span>
            <span class="inline-label input-label" v-else>{{ t('main.keys.key') }}: </span>
            <CompleteInput
                v-model="newKeyDialog.key"
                :search-func="searchNextDir"
                density="comfortable"
                prepend-inner-icon="mdi-file-document"
                :prefix="session.namespace"
                :hint="t('main.keys.completeInputHint')"
                persistent-hint
                elevation="16"
            />
          </v-layout>
          <v-layout class="mb-5" style="z-index: unset">
            <span class="inline-label radio-label"></span>
            <v-radio-group v-model="newKeyDialog.model" inline hide-details>
              <v-radio :label="t('main.keys.neverExpire')" value="none"/>
              <v-radio :label="t('main.keys.withTtl')" value="ttl"/>
              <v-radio :label="t('main.keys.withLease')" value="lease"/>
            </v-radio-group>
          </v-layout>
          <v-layout class="mb-5" style="z-index: unset" v-if="newKeyDialog.model == 'ttl'">
            <span class="inline-label input-label">{{ t('common.ttl') }}(s): </span>
            <v-text-field
                v-model="newKeyDialog.ttl"
                type="number"
                density="comfortable"
                prepend-inner-icon="mdi-clock-time-eight"
                :hint="t('main.keys.ttlHint')"
                persistent-hint
            />
          </v-layout>
          <v-layout class="mb-5" style="z-index: unset" v-if="newKeyDialog.model == 'lease'">
            <span class="inline-label input-label">{{t('common.lease')}}: </span>
            <v-text-field
                v-model="newKeyDialog.lease"
                type="number"
                density="comfortable"
                prepend-inner-icon="mdi-identifier"
                :hint="t('main.keys.leaseHint')"
                persistent-hint
            />
          </v-layout>
          <div style="height: 40vh;width:100%">
            <editor ref="newKeyEditorRef" :value="newKeyDialog.value" :config="newKeyEditorConfig"/>
          </div>
        </v-card-text>
        <v-card-actions>
          <v-btn
              :text="t('common.cancel')"
              variant="text"
              class="text-none"
              @click="newKeyDialog.show = false"
          />

          <v-btn
              :text="t('common.commit')"
              variant="flat"
              class="text-none"
              color="primary"
              @click="putKey"
              :loading="loadingStore.confirmNewKey"
          />
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!--   key收藏弹窗-->
    <v-dialog v-model="collectionDialog"
              eager
              transition="slide-x-reverse-transition"
              scrollable
              class="collection-drawer-right"
              contained
    >
      <v-card :rounded="false" :title="t('main.keys.myCollections')">
        <template #prepend>
          <v-icon color="#ced10a">mdi-star</v-icon>
        </template>
        <v-card-item style="height: calc(100% - 64px);">
          <div class="mx-2">
            <CompleteInput
                v-model="addCollectionKeyForm"
                append-inner-icon="mdi-plus"
                density="compact"
                variant="solo-filled"
                hide-details
                single-line
                clearable
                :placeholder="t('main.keys.collectionInputPlaceholder')"
                :search-func="searchNextNode"
                @click:appendInner="addCollectionKey(addCollectionKeyForm, true); addCollectionKeyForm = '';"
            />
          </div>
          <div class="overflow-y-auto full-width" style="height: calc(100% - 40px);">
            <Tree ref="kvCollectionTree"
                  :tree-id="`kv-collection-tree-${new Date().getTime()}`"
                  :key-splitter="KEY_SPLITTER"
                  :session="session"
                  :show-node-suffix="false"
                  :show-check-box="false"
                  show-hover-remove
                  :enable-select="false"
                  style="height: 100%;"
                  :init-items="session.keyCollection"
                  @on-click="onClickKeyCollectionTreeItem"
                  @on-click-remove="removeCollectionKey"
            />
          </div>
        </v-card-item>
      </v-card>
    </v-dialog>

    <!--   服务器搜索弹窗-->
    <v-dialog v-model="searchDialog.show" max-width="800px" scrollable>
      <v-card>
        <v-card-title class="pa-0">
          <v-text-field v-model="searchDialog.inputValue"
                        :prefix="session.namespace"
                        autofocus
                        type="text"
                        @input="searchFromServer"
                        :placeholder="t('main.keys.searchFromServerPlaceHolder')"
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
              />
            </template>
          </v-text-field>
        </v-card-title>
        <v-card-text class="pa-0">
          <v-list lines="two"
                  v-if="searchDialog.searchResult && searchDialog.searchResult.results.length > 0"
          >
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
                {{ t('common.version') }}: <i>{{ kv.version }}</i>,
                {{ t('main.keys.createRevision') }}: <i>{{ kv.createRevision }}</i>,
                {{ t('main.keys.modifyRevision') }}: <i>{{ kv.modRevision }}</i>
              </template>
            </v-list-item>
          </v-list>
        </v-card-text>
        <v-card-actions class="text-medium-emphasis">
          <v-spacer/>
          <span v-if="searchDialog.searchResult">{{ t('main.keys.searched') }} {{
              searchDialog.searchResult.results.length
            }} / {{ searchDialog.searchResult.count }}</span>
          <span v-else>{{ t('main.keys.searchFromServerTip') }}</span>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!--   更新冲突Merge弹窗-->
    <v-dialog v-model="putMergeDialog.show" max-width="1200px" min-width="800px" persistent scrollable>
      <v-card :title="t('main.keys.resolveConflict')">
        <v-card-text>
          <v-alert type="warning"
                   :text="t('main.keys.resolveConflictAlert')"
                   class="my-2"
                   density="compact"
          />
          <v-row class="my-2">
            <v-col cols="6" class="text-center font-weight-bold text-medium-emphasis">
              {{ t('main.keys.yourVersion') }}
            </v-col>
            <v-col cols="6" class="text-center font-weight-bold text-medium-emphasis">
              {{ t('main.keys.latestVersion', {version: putMergeDialog.existVersion}) }}
            </v-col>
          </v-row>
          <div ref="putMergeEditorRef"/>
        </v-card-text>

        <v-card-actions>
          <v-btn
              :text="t('common.cancel')"
              variant="text"
              class="text-none"
              @click="cancelMergeDialog"
          />

          <v-btn
              :text="t('main.keys.resolvedAndSubmit')"
              variant="flat"
              class="text-none"
              color="primary"
              @click="confirmMergeDialog"
          />
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!--   重命名目录-->
    <v-dialog
        v-model="renameDirDialog.show"
        max-width="800px"
        scrollable
        persistent
    >
      <v-card :title="t('main.keys.renamePath')">
        <v-card-text class="rename-form">
          <v-layout class="mb-5">
            <span class="inline-label input-label">{{t('main.keys.path')}}: </span>
            <v-text-field
                v-model="renameDirDialog.newPrefix"
                :prefix="session.namespace"
                density="comfortable"
                prepend-inner-icon="mdi-file-document"
                hide-details
            />
          </v-layout>
          <v-layout class="mb-5">
            <span class="inline-label checkbox-label"></span>
            <v-checkbox
                v-model="renameDirDialog.deleteOriginKeys"
                :label="t('main.keys.deleteOriginKeys')"
                hide-details
            />
          </v-layout>
          <v-layout class="mb-5">
            <span class="inline-label radio-label" style="line-height: 40px;">{{t('main.keys.putStrategy')}}: </span>
            <v-radio-group v-model="renameDirDialog.putStrategy"
                           inline
                           hide-details
                           style="flex-direction: row;">
              <v-radio :label="t('main.keys.coverStrategy')" value="Cover"/>
              <v-radio :label="t('main.keys.renameStrategy')" value="Rename"/>
            </v-radio-group>
          </v-layout>

          <div v-if="renameDirDialog.state != 'none'">
            <v-divider v-if="renameDirDialog.logs.length > 0">{{ t('main.keys.logs')}} </v-divider>
            <div style="max-height: 30vh;" class="overflow-auto" ref="renameDirLogListRef">
              <div v-for="(log, idx) in renameDirDialog.logs" :key="idx">
                <div v-if="log.success">
                  [<strong style="color: #4CAF50;">{{t('common.success')}}</strong>]
                  <span style="color: #00BCD4;">{{log.action}}</span>
                  {{_decodeBytesToString(log.key)}}
                </div>
                <div v-else>
                  <p>
                    [<strong style="color: #E57373;">{{t('common.failed')}}</strong>]
                    <span style="color: #00BCD4;">{{log.action}}</span>
                    {{_decodeBytesToString(log.key)}}
                  </p>
                  <p style="color: #E57373;">{{log.failedMsg}}</p>
                </div>
              </div>
            </div>
          </div>
        </v-card-text>
        <v-card-actions>
          <v-btn
              :text="renameDirDialog.state == 'none' ? t('common.cancel') : t('common.close')"
              variant="text"
              class="text-none"
              :disabled="renameDirDialog.state == 'started'"
              @click="() => {renameDirDialog.show = false;renameDirDialog.logs=[];}"
          />

          <v-btn
              :text="t('common.commit')"
              variant="flat"
              class="text-none"
              color="primary"
              @click="renameDir"
              v-if="renameDirDialog.state == 'none' || renameDirDialog.state == 'started'"
              :disabled="renameDirDialog.state == 'started'"
              :loading="loadingStore.renameDir"
          />
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

.inline-label {
  width: 80px;
}

.rename-form {
  .inline-label {
    width: 120px;
  }
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
