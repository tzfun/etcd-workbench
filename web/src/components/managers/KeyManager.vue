<script setup lang="ts">
import {
  _copyAndSave,
  _deleteKey,
  _exportKeys,
  _getAllKeys,
  _getAllKeysPaging,
  _getKV,
  _getKVHistory,
  _importKeys,
  _putKV
} from "~/common/Service";
import {
  BottomRight,
  Delete,
  DocumentAdd,
  FolderOpened,
  List,
  Refresh,
  Switch,
  TopRight,
  UploadFilled
} from "@element-plus/icons-vue";
import {EditorConfig, KeyDTO, KeyValueDTO, TreeNode} from "~/common/Types";
import Editor from "~/components/editor/Editor.vue";
import {reactive} from "vue";
import {_endLoading, _isEmpty, _parseCodeLanguage, _saveFile, _startLoading} from "~/common/Util";
import {CodeDiff} from "v-code-diff";
import KeyTableViewer from "~/components/viewer/KeyTableViewer.vue";
import KeyTreeViewer from "~/components/viewer/KeyTreeViewer.vue";
import {isDark} from "~/composables";
import {ElMessage, UploadUserFile} from "element-plus";

const editorRef = ref()
const treeViewerRef = ref<InstanceType<typeof KeyTreeViewer>>()
const tableViewerRef = ref<InstanceType<typeof KeyTableViewer>>()
const props = defineProps({
  sessionKey: {
    type: String,
    required: true
  }
})
const drawer = ref(false)
const KEY_SPLITTER = "/"
const KEY_PAGING_LIMIT = 5000

const viewer = ref<'tree' | 'table'>('tree')
const loadedViewer = reactive({
  tree: true,
  table: false
})
const tableData = ref<KeyDTO[]>([])
const treeData = reactive<TreeNode>({
  path: KEY_SPLITTER,
  type: "dir",
  children: [],
  label: ''
})
const paginationKeyCursor = ref<string | undefined>("")

const editing = ref<boolean>(false)
const isNew = ref<boolean>(false)
const editingKV = ref<KeyValueDTO>()

const showDiff = ref<boolean>(false)
const versionDiffInfo = reactive({
  key: '',
  version: 0,
  createRevision: 0,
  modRevision: 0,
  versionHistory: [],
  versionA: 0,
  versionAContent: '',
  versionB: 0,
  versionBContent: ''
})

const showCopyAndSave = ref<boolean>(false)
const copyAndSaveForm = reactive({
  src: <string | null>null,
  dest: <string | null>null,
  ttl: 0
})
const importKeysFile = ref<UploadUserFile[]>()

onMounted(() => {
  refreshAllKeys()
})

const refreshAllKeys = () => {
  treeData.children = []
  tableData.value = []
  paginationKeyCursor.value = ""
  loadNextPage()
}

const loadAllKeys = () => {
  _getAllKeys(props.sessionKey as string).then((data: KeyValueDTO[]) => {
    tableData.value = data
    addKvListToTree(data)
    if (viewer.value === 'tree') {
      treeViewerRef.value!.clear()
    }
  })
}

const loadNextPage = () => {
  let cursor = paginationKeyCursor.value
  if (cursor != undefined) {
    _startLoading()
    _getAllKeysPaging(props.sessionKey, cursor, KEY_PAGING_LIMIT).then((data: KeyValueDTO[]) => {
      if (data.length < KEY_PAGING_LIMIT) {
        paginationKeyCursor.value = undefined
      }

      if (data.length > 0) {
        if (paginationKeyCursor.value != undefined) {
          paginationKeyCursor.value = data[data.length - 1].key
        }

        addKvListToTree(data)
        tableData.value.push(...data)
      }
    }).catch(e => {
      console.log(e)
    }).finally(() => {
      _endLoading()
    })
  }
}

const addKvListToTree = (data: KeyValueDTO[]) => {
  for (let kv of data) {
    addKvToTree(kv)
  }
}

const switchViewer = (v: string) => {
  if (!loadedViewer[v]) {
    loadedViewer[v] = true
  }
}

const addKvToTree = (kv: KeyValueDTO) => {
  let k = kv.key
  let originKey = kv.key
  //  为了方便解析为统一的树状结构，如果key不是以分隔符开头，默认补充分隔符
  if (!k.startsWith(KEY_SPLITTER)) {
    k = KEY_SPLITTER + k
  }

  let splits = k.split(KEY_SPLITTER)
  let node: TreeNode = treeData
  const path = [""]
  //  只遍历路径
  for (let i = 1; i < splits.length - 1; i++) {
    const floorName = splits[i]
    let floorNode: TreeNode
    path.push(floorName)
    let found = false
    for (let treeNode of node.children) {
      if (treeNode.type === 'dir' && treeNode.label === floorName) {
        found = true
        floorNode = treeNode
        break
      }
    }
    if (!found) {
      floorNode = {
        path: "@" + path.join(KEY_SPLITTER),
        type: 'dir',
        label: floorName,
        children: <TreeNode>[]
      }
      node.children?.push(floorNode)
    }
    node = floorNode
  }
  let fileName = splits[splits.length - 1]
  let fileNode: TreeNode = {
    path: originKey,
    type: 'file',
    label: fileName,
    data: kv
  }
  node.children?.push(fileNode)
}

const editorConfig = reactive<EditorConfig>({
  disabled: false,
  indentWithTab: true,
  tabSize: 2,
  autofocus: true,
  height: "50vh",
  fontSize: "1rem",
  language: 'json'
})

const add = () => {
  editingKV.value = {
    key: '',
    value: '',
    ttl: null
  }
  editorConfig.language = 'text'
  isNew.value = true
  editing.value = true
}

const edit = (info: KeyDTO, index: number) => {

  _getKV(props.sessionKey, info.key).then(data => {
    if (data) {
      editingKV.value = data
      editorConfig.language = _parseCodeLanguage(info.key, data.value)

      isNew.value = false
      editing.value = true
    } else {
      ElMessage({
        showClose: true,
        message: "The key does not exist or has expired.",
        type: 'info',
      })
      tableData.value.splice(index, 1)
    }
  })
}

const getKVDetail = ({key, callback}) => {
  _getKV(props.sessionKey, key).then(data => {
    if (!data) {
      deleteKeysFromTree([key])
    }
    callback(data)
  })
}

const diff = (row: KeyDTO) => {
  _getKV(props.sessionKey, row.key).then((dataB: KeyValueDTO) => {
    if (dataB) {
      versionDiffInfo.key = dataB.key
      versionDiffInfo.createRevision = dataB.createRevision
      versionDiffInfo.modRevision = dataB.modRevision
      versionDiffInfo.versionB = row.modRevision
      versionDiffInfo.versionBContent = dataB.value
      versionDiffInfo.version = dataB.version
      versionDiffInfo.modRevision = dataB.modRevision

      _getKVHistory(
          props.sessionKey,
          versionDiffInfo.key,
          versionDiffInfo.createRevision,
          versionDiffInfo.modRevision
      ).then((versions: number[]) => {
        if (versions.length <= 1) {
          ElMessage({
            type: 'info',
            message: 'No multiple versions',
          })
          return
        }
        versionDiffInfo.versionHistory = versions

        //  上一个版本
        versionDiffInfo.versionA = versionDiffInfo.versionHistory[dataB.version - 2]
        loadDiff(true)
      }).catch(e => {
        console.error(e)
      })
    } else {
      ElMessage({
        showClose: true,
        message: "The key does not exist or has expired.",
        type: 'info',
      })
    }
  }).catch(e => {
    console.error(e)
  })




}

const loadDiff = (forA: Boolean) => {
  const queryVersion = forA ? versionDiffInfo.versionA : versionDiffInfo.versionB

  _getKV(props.sessionKey, versionDiffInfo.key, queryVersion).then(data => {
    let queryValue = '';
    if (data == null) {
      ElMessage({
        type: 'warning',
        message: `History not found with version: ${queryVersion}`,
      })
    } else {
      queryValue = data.value
    }
    if (forA) {
      versionDiffInfo.versionAContent = queryValue
    } else {
      versionDiffInfo.versionBContent = queryValue
    }
    if (!showDiff.value) {
      showDiff.value = true
    }
  }).catch(e => {
    console.error(e)
  })
}

const del = ({key, callback}) => {
  ElMessageBox.confirm(
      `Are you sure to delete this key? <br><strong>${key}</strong>`,
      'Confirm',
      {
        confirmButtonText: 'OK',
        cancelButtonText: 'Cancel',
        dangerouslyUseHTMLString: true,
        type: 'warning',
      }
  ).then(() => {
    _deleteKey(props.sessionKey, [key]).then(() => {
      ElMessage({
        type: 'success',
        message: 'Deleted successfully',
      })

      if (callback) {
        callback(key)
      }
      deleteKeysFromTree([key])
      tableViewerRef.value!.deleteKey(key)
    }).catch(e => {
      console.error(e)
    })
  }).catch(() => {
  })
}

const delBatch = () => {
  let keys: string[]
  if (viewer.value === 'tree') {
    keys = treeViewerRef.value!.getSelectedKeys()
  } else {
    keys = tableViewerRef.value!.getSelectedKeys()
  }

  if (keys.length == 0) {
    ElMessage({
      type: 'info',
      message: 'No selected keys',
    })
    return
  }

  ElMessageBox.confirm(
      'Are you sure to delete this keys?',
      'Confirm',
      {
        confirmButtonText: 'OK',
        cancelButtonText: 'Cancel',
        type: 'warning',
      }
  ).then(() => {
    _deleteKey(props.sessionKey, keys).then(() => {
      ElMessage({
        type: 'success',
        message: 'Deleted successfully',
      })
      //  删除tree试图
      treeViewerRef.value!.clearSelectedKeys()
      deleteKeysFromTree(keys)

      //  删除table视图
      tableViewerRef.value!.clearSelectedKeys()
      tableData.value = tableData.value.filter(item => !keys.includes(item.key))
    }).catch(e => {
      console.error(e)
    })
  }).catch(() => {
  })
}

const exportKeys = () => {
  let keys
  if (viewer.value === 'tree') {
    keys = treeViewerRef.value!.getSelectedKeys()
  } else {
    keys = tableViewerRef.value!.getSelectedKeys()
  }

  if (keys.length == 0) {
    ElMessage({
      type: 'info',
      message: 'No selected keys',
    })
    return
  }
  _startLoading("Search selected keys...")
  _exportKeys(props.sessionKey, keys).then((data) => {
    const blob = new Blob([data], {type: 'plain/text'});
    _saveFile(blob, "keys-dump.ew")
  }).finally(() => {
    _endLoading()
  })
}

const importKeys = () => {
  const file = importKeysFile.value[0]
  if (file) {
    (file.raw as File).text().then(data => {
      ElMessageBox.confirm(
          `Are you sure want to import key/value(s) from file \'${file.name}\'?`,
          'Confirm',
          {
            confirmButtonText: 'Yes',
            cancelButtonText: 'Cancel',
            type: 'info',
          }
      ).then(() => {
        _importKeys(props.sessionKey, data).then(() => {
          drawer.value = false
          importKeysFile.value = []
          refreshAllKeys()
          ElMessage({
            showClose: true,
            message: "Import success",
            type: "success"
          })
        })
      }).catch(() => {
      })
    }).catch(e => {
      ElMessage({
        showClose: true,
        message: e.message,
        type: "error",
        duration: 5000,
      })
    })
  }
}

const onCopyAndSave = (key: string) => {
  copyAndSaveForm.src = key
  copyAndSaveForm.dest = null

  showCopyAndSave.value = true
}

const deleteKeysFromTree = (keys: string[]) => {
  for (let key of keys) {
    let keyArr = key.split(KEY_SPLITTER)
    let i = 1;
    let stack = []
    let nodeArr = treeData.children
    while (nodeArr && nodeArr.length > 0 && i < keyArr.length) {
      let label = keyArr[i]
      let isFinal = i === keyArr.length - 1
      for (let node of nodeArr) {
        if (!isFinal && node.type == 'file') {
          continue
        }
        if (node.label === label) {
          stack.push(node)
          nodeArr = node.children
          i++
          break
        }
      }
    }

    if (stack.length === 0) {
      let j = 0
      for (; j < treeData.children.length; j++) {
        let node = treeData.children[j]
        if (node.type == 'file' && node.path === key) {
          break
        }
      }
      treeData.children.splice(j, 1)
      continue
    }

    let node
    let parent
    do {
      let newNode = stack.pop()
      if (!newNode.children || newNode.children.length == 1) {
        node = newNode
      } else {
        parent = newNode.children
        break
      }
    } while (stack.length > 0 && node)

    if (!parent) {
      parent = treeData
    }
    let idx = parent.indexOf(node)
    if (idx >= 0) {
      parent.splice(idx, 1)
    }
  }
}

const tablePutKey = () => {
  const kv: KeyValueDTO = editingKV.value as KeyValueDTO
  const key = kv.key
  if (_isEmpty(key)) {
    ElMessage({
      type: 'warning',
      message: 'Key is empty',
    })
    return
  }

  const value = editorRef.value.readDataString()

  if (!isNew.value) {
    if (value == kv.value) {
      ElMessage({
        type: 'warning',
        message: 'Content not change',
      })
      return
    }
  }

  putKeyValue({
    kv: {
      ...kv,
      value: value
    },
    callback: null
  })
}

const putKeyValue = ({kv, callback}) => {
  _putKV(props.sessionKey, kv.key, kv.value, kv.ttl).then((data: KeyValueDTO) => {
    if (callback) {
      callback()
    }
    //  新建
    if (data.version === 1) {
      tableData.value.push(data)
      addKvToTree(data)
    } else {
      for (let i = 0; i < tableData.value.length; i++) {
        if (tableData.value[i].key === data.key) {
          tableData.value[i] = data
          break
        }
      }
    }
    editing.value = false
  }).catch(e => {
    console.error(e)
  })
}

const confirmCopyAndSave = () => {
  if (copyAndSaveForm.ttl < 0) {
    ElMessage({
      type: 'warning',
      message: 'Invalid TTL',
    })
    return
  }

  if (_isEmpty(copyAndSaveForm.dest)) {
    ElMessage({
      type: 'warning',
      message: 'Target key cannot be empty',
    })
    return
  }

  if (copyAndSaveForm.src == copyAndSaveForm.dest) {
    ElMessage({
      type: 'warning',
      message: 'From key and To key cannot be the same',
    })
    return
  }

  console.debug("copy", copyAndSaveForm.src, "to", copyAndSaveForm.dest, "in ttl", copyAndSaveForm.ttl)
  _copyAndSave(
      props.sessionKey,
      copyAndSaveForm.src as string,
      copyAndSaveForm.dest as string,
      copyAndSaveForm.ttl
  ).then((data: KeyValueDTO) => {
    //  新建
    if (data.version === 1) {
      tableData.value.push(data)
      addKvToTree(data)
    }
    showCopyAndSave.value = false
  }).catch(e => {
    console.error(e)
  })
}

</script>

<template>
  <div class="page">
    <div class="button-list">
      <el-button :icon="Refresh" @click="refreshAllKeys" size="default">Refresh</el-button>
      <el-button type="primary" :icon="DocumentAdd" @click="add">Add Key</el-button>
      <el-button type="danger" :icon="Delete" @click="delBatch">Delete Keys</el-button>
      <el-button type="success" plain :icon="TopRight" @click="exportKeys">Export Keys</el-button>
      <el-button type="warning" plain :icon="BottomRight" @click="drawer=true">Import Keys</el-button>

      <el-switch
          v-model="viewer"
          @change="switchViewer"
          size="large"
          active-value="tree"
          inactive-value="table"
          inline-prompt
          :active-action-icon="Switch"
          :inactive-action-icon="Switch"
          :active-icon="List"
          :inactive-icon="FolderOpened"
          style="margin-left: auto;--ep-switch-off-color: #a59cf1;"
          title="Switch viewer type"
      />

    </div>

    <div class="tree-viewer"
         v-show="viewer === 'tree'"
    >
      <key-tree-viewer ref="treeViewerRef"
                       v-if="loadedViewer.tree"
                       :data="treeData.children"
                       :has-more-data="paginationKeyCursor != undefined"
                       @on-select="getKVDetail"
                       @on-save="putKeyValue"
                       @on-diff="diff"
                       @on-delete="del"
                       @copy-and-save="onCopyAndSave"
                       @load-more="loadNextPage"/>
    </div>
    <div class="table-viewer"
         v-show="viewer === 'table'"
    >
      <key-table-viewer ref="tableViewerRef"
                        v-if="loadedViewer.table"
                        :data="tableData"
                        :has-more-data="paginationKeyCursor != undefined"
                        @on-edit="edit"
                        @on-diff="diff"
                        @on-delete="del"
                        @copy-and-save="onCopyAndSave"
                        @load-more="loadNextPage"/>
    </div>

    <!-- 编辑弹窗 -->
    <el-dialog v-model="editing"
               title="Key Editor"
               :close-on-click-modal="false"
               append-to-body
               align-center>
      <el-row :gutter="20" class="mt-2 mb-2">
        <span class="editor-label">Key:</span>
        <el-input v-model="(editingKV as KeyValueDTO).key"
                  class="inline-flex editor-input"
                  placeholder="Input key"
                  :disabled="!isNew"></el-input>
      </el-row>
      <el-row :gutter="20" class="mt-2 mb-2" v-if="isNew">
        <span class="editor-label">TTL(s):</span>
        <el-input-number v-model="(editingKV as KeyValueDTO).ttl"
                         class="inline-flex"
                         style="width: 300px"
                         placeholder="Key expiration time, in seconds"/>
      </el-row>
      <editor ref="editorRef"
              :key="editingKV"
              :code="(editingKV as KeyValueDTO).value"
              :config="editorConfig"/>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="editing = false">Cancel</el-button>
          <el-button type="primary" @click="tablePutKey">
            Confirm
          </el-button>
        </span>
      </template>
    </el-dialog>

    <!-- Diff 弹窗 -->
    <el-dialog v-model="showDiff"
               :title="`Version Diff: ${versionDiffInfo.key}`"
               :close-on-click-modal="false"
               append-to-body
               align-center>
      <div class="flex items-center">
        <div class="diff-label">Version A:</div>
        <el-select v-model="versionDiffInfo.versionA"
                   fit-input-width
                   class="diff-select"
                   placeholder="Select language"
                   @change="loadDiff(true)">
          <el-option
              v-for="item in versionDiffInfo.versionHistory"
              :key="item"
              :label="item"
              :value="item"
          >
            <span style="float: left">{{ item }}</span>
            <span v-if="item == versionDiffInfo.modRevision" class="version-option-tag">
              latest
            </span>
            <span v-else-if="item == versionDiffInfo.createRevision" class="version-option-tag">
              create
            </span>
          </el-option>
        </el-select>

        <div class="diff-label" style="margin-left: auto">Version B:</div>
        <el-select v-model="versionDiffInfo.versionB"
                   fit-input-width
                   class="diff-select"
                   placeholder="Select language"
                   @change="loadDiff(false)">
          <el-option
              v-for="item in versionDiffInfo.versionHistory"
              :key="item"
              :label="item"
              :value="item"
          >
            <span style="float: left">{{ item }}</span>
            <span v-if="item == versionDiffInfo.modRevision" class="version-option-tag">
                latest
              </span>
            <span v-else-if="item == versionDiffInfo.createRevision" class="version-option-tag">
                create
              </span>
          </el-option>
        </el-select>
      </div>


      <code-diff
          style="max-height: 70vh;min-height:50vh;"
          :old-string="versionDiffInfo.versionAContent"
          :filename="`Revision: ${versionDiffInfo.versionA}`"
          :new-string="versionDiffInfo.versionBContent"
          :new-filename="`Revision: ${versionDiffInfo.versionB}`"
          :theme="isDark ? 'dark' : 'light'"
          output-format="side-by-side"/>
    </el-dialog>

    <!-- 复制保存弹窗 -->
    <el-dialog v-model="showCopyAndSave"
               title="Copy And Save"
               :close-on-click-modal="false"
               append-to-body
               align-center>
      <el-row :gutter="20" class="mt-2 mb-2">
        <span class="editor-label">From:</span>
        <el-input v-model="copyAndSaveForm.src"
                  class="inline-flex editor-input"
                  readonly/>
      </el-row>
      <el-row :gutter="20" class="mt-2 mb-2">
        <span class="editor-label">To:</span>
        <el-input v-model="copyAndSaveForm.dest"
                  class="inline-flex editor-input"
                  placeholder="Copy value to target key"/>
      </el-row>
      <el-row :gutter="20" class="mt-2 mb-2">
        <span class="editor-label">TTL(s):</span>
        <el-input-number v-model="copyAndSaveForm.ttl"
                         class="inline-flex"
                         style="width: 300px"
                         placeholder="Key expiration time, in seconds"/>
      </el-row>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="showCopyAndSave = false">Cancel</el-button>
          <el-button type="primary" @click="confirmCopyAndSave">
            Confirm
          </el-button>
        </span>
      </template>
    </el-dialog>

    <el-drawer v-model="drawer"
               title="Select import file"
               :with-header="false"
               append-to-body
    >
      <el-alert
          title="If there are already existing parts in the imported keys, these parts will be forcibly overwritten, and non-existing keys will be inserted."
          type="warning"
          :closable="false"
          class="mb-2"/>
      <el-upload
          class="upload-demo"
          drag
          :auto-upload="false"
          accept=".ew"
          :limit="1"
          v-model:file-list="importKeysFile"
      >
        <el-icon class="el-icon--upload">
          <UploadFilled/>
        </el-icon>
        <div class="el-upload__text">
          Drop file here or <em>click to select</em>
        </div>
        <template #tip>
          <div class="el-upload__tip">
            Only supports files in <em>.ew</em> format.
          </div>
        </template>
      </el-upload>
      <el-button type="success" @click="importKeys" class="mt-2">Upload</el-button>
    </el-drawer>
  </div>
</template>

<style lang="scss" scoped>
@import '../../styles/index.scss';

.version-option-tag {
  float: right;
  color: #909399FF;
  font-size: 13px;
}

.page {
  $--button-list-height: 30px;
  $--button-list-margin-bottom: 15px;

  .tree-viewer {
    height: calc(100% - $--button-list-height - $--button-list-margin-bottom - 2px);
  }

  .button-list {
    height: calc($--button-list-height + $--button-list-margin-bottom);
  }
}

$--editor-label-width: 60px;

.editor-label {
  width: $--editor-label-width;
  text-align: center;
  line-height: 30px;
}

.editor-input {
  width: calc(100% - $--editor-label-width);
}

.diff-label {
  width: 70px;
  font-size: 0.9em;
  color: #168f8f;
  font-weight: bold;
}

.diff-select {
  width: 200px;
}
</style>
