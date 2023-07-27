<script setup lang="ts">
import {deleteKey, getAllKeys, getKV, getKVHistory, putKV} from "~/services/SessionService";
import {Delete, DocumentAdd, DocumentCopy, Edit, Refresh, Search} from "@element-plus/icons-vue";
import {EditorConfig, KeyDTO, KeyValueDTO} from "~/entitys/TransformTypes";
import Editor from "~/components/editor/Editor.vue";
import {isDark} from "~/composables";
import {reactive} from "vue";
import {_isEmpty} from "~/util/Util";
import {CodeDiff} from "v-code-diff";
import {Base64} from 'js-base64';

const editorRef = ref(null)
const props = defineProps({
  sessionKey: {
    type: String,
    required: true
  }
})

onMounted(() => {
  loadAllKeys()
})

const tableData = ref<Array<KeyDTO>>([])
const filterTableData = computed(() =>
    tableData.value.filter(
        (data) =>
            !keySearch.value ||
            data.key.toLowerCase().includes(keySearch.value.toLowerCase())
    )
)
const selectedKey = ref<string[]>([])
const keySearch = ref()
const editing = ref<Boolean>(false)
const isNew = ref<Boolean>(false)
const editingKV = ref<KeyValueDTO>()

const editorConfig = reactive<EditorConfig>({
  disabled: false,
  indentWithTab: true,
  tabSize: 2,
  autofocus: true,
  height: "50vh",
  language: 'json',
  theme: isDark ? 'oneDark' : 'default'
})

const showDiff = ref<Boolean>(false)
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

const handleSelectionChange = (rows: KeyValueDTO[]) => {
  let selected = []
  for (let row of rows) {
    selected.push(row.key)
  }
  selectedKey.value = selected
}

const loadAllKeys = () => {
  getAllKeys(props.sessionKey as string).then(data => {
    tableData.value = data
  })
}

const add = () => {
  editingKV.value = {
    key: '',
    value: ''
  }
  editorConfig.language = 'text'
  isNew.value = true
  editing.value = true
}

const edit = (index, row: KeyDTO) => {
  getKV(props.sessionKey, row.key).then(data => {
    editingKV.value = data
    const content = data.value
    if (content.startsWith('<')) {
      editorConfig.language = 'xml'
    } else if (content.startsWith('{') || content.startsWith('[')) {
      editorConfig.language = 'json'
    } else if (content.startsWith('---')) {
      editorConfig.language = 'yaml'
    } else {
      editorConfig.language = 'text'
    }

    isNew.value = false
    editing.value = true
  })
}

const diff = (index, row: KeyDTO) => {
  if (row.version <= 1) {
    ElMessage({
      type: 'info',
      message: 'No multiple versions',
    })
    return
  }

  versionDiffInfo.version = row.version
  versionDiffInfo.key = row.key
  versionDiffInfo.createRevision = row.createRevision
  versionDiffInfo.modRevision = row.modRevision

  getKVHistory(
      props.sessionKey,
      versionDiffInfo.key,
      versionDiffInfo.createRevision,
      versionDiffInfo.modRevision).then(data => {
    versionDiffInfo.versionHistory = data.sort()
    getKV(props.sessionKey, versionDiffInfo.key).then(data => {
      versionDiffInfo.versionB = row.modRevision
      versionDiffInfo.versionBContent = data.value

      //  上一个版本
      versionDiffInfo.versionA = versionDiffInfo.versionHistory[row.version - 2]
      loadDiff(true)
    }).catch(e => {
      console.error(e)
    })
  }).catch(e => {
    console.error(e)
  })
}

const loadDiff = (forA: Boolean) => {
  const queryVersion = forA ? versionDiffInfo.versionA : versionDiffInfo.versionB

  getKV(props.sessionKey, versionDiffInfo.key, queryVersion).then(data => {
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

const del = (index, row: KeyDTO) => {
  ElMessageBox.confirm(
      `Are you sure to delete this key? <br><strong>${row.key}</strong>`,
      'Confirm',
      {
        confirmButtonText: 'OK',
        cancelButtonText: 'Cancel',
        dangerouslyUseHTMLString: true,
        type: 'warning',
      }
  ).then(() => {
    deleteKey(props.sessionKey, [row.key]).then(() => {
      ElMessage({
        type: 'success',
        message: 'Deleted successfully',
      })
      tableData.value.splice(index, 1)
    }).catch(e => {
      console.error(e)
    })
  }).catch(() => {
  })
}

const delBatch = () => {
  if (selectedKey.value.length == 0) {
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
    deleteKey(props.sessionKey, selectedKey.value).then(() => {
      ElMessage({
        type: 'success',
        message: 'Deleted successfully',
      })
      selectedKey.value = []
      loadAllKeys()
    }).catch(e => {
      console.error(e)
    })
  }).catch(() => {
  })
}

const putKey = () => {
  const key = editingKV.value.key
  if (_isEmpty(key)) {
    ElMessage({
      type: 'warning',
      message: 'Key is empty',
    })
    return
  }

  const value = editorRef.value.code

  if (_isEmpty(value)) {
    ElMessage({
      type: 'warning',
      message: 'Content value is empty',
    })
    return
  }

  if (value == editingKV.value.value) {
    ElMessage({
      type: 'warning',
      message: 'Content not change',
    })
    return
  }

  putKV(props.sessionKey, key, value).then(() => {
    loadAllKeys()
    editing.value = false
  }).catch(e => {
    console.error(e)
  })
}
</script>

<template>
  <div class="mb-5">
    <el-button :icon="Refresh" @click="loadAllKeys">Refresh Table</el-button>
    <el-button type="primary" :icon="DocumentAdd" @click="add">Add Key / Value</el-button>
    <el-button type="danger" :icon="Delete" @click="delBatch">Delete Keys</el-button>
  </div>

  <el-table :data="filterTableData"
            border
            stripe
            @selection-change="handleSelectionChange"
            class="mb-10">
    <el-table-column type="selection" width="55"/>
    <el-table-column prop="key" label="Key" sortable/>
    <el-table-column prop="version" label="Version" sortable/>
    <el-table-column prop="createRevision" label="Create Revision" sortable/>
    <el-table-column prop="modRevision" label="Modify Revision" sortable/>
    <el-table-column prop="lease" label="Lease"/>
    <el-table-column fixed="right" label="Operations" width="300">
      <template #header>
        <el-input v-model="keySearch" placeholder="Type to search" :prefix-icon="Search"/>
      </template>
      <template #default="scope">
        <el-button type="primary" :icon="Edit" plain size="small" @click="edit(scope.$index,scope.row)">Edit</el-button>
        <el-button type="info" :icon="DocumentCopy" plain size="small" @click="diff(scope.$index,scope.row)">Version
          Diff
        </el-button>
        <el-button type="danger" :icon="Delete" size="small" @click="del(scope.$index,scope.row)">Delete</el-button>
      </template>
    </el-table-column>
  </el-table>

  <el-dialog v-model="editing"
             title="Key Editor"
             :close-on-click-modal="false"
             align-center>
    <el-row :gutter="20" class="mt-2 mb-2">
      <span style="width: 60px;text-align: center;line-height: 30px;">Key:</span>
      <el-input v-model="editingKV.key"
                class="inline-flex"
                style="width: calc(100% - 60px)"
                :disabled="!isNew"></el-input>
    </el-row>
    <editor ref="editorRef"
            :key="editingKV"
            :code="editingKV.value"
            :config="editorConfig"/>
    <template #footer>
      <span class="dialog-footer">
        <el-button @click="editing = false">Cancel</el-button>
        <el-button type="primary" @click="putKey">
          Confirm
        </el-button>
      </span>
    </template>
  </el-dialog>

  <el-dialog v-model="showDiff"
             :title="`Version Diff: ${versionDiffInfo.key}`"
             :close-on-click-modal="false"
             align-center>
    Version A:
    <el-select v-model="versionDiffInfo.versionA"
               fit-input-width
               class="inline-flex"
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

    <div style="float: right">
      Version B:
      <el-select v-model="versionDiffInfo.versionB"
                 fit-input-width
                 class="inline-flex"
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
        :new-string="versionDiffInfo.versionBContent"
        :file-name="versionDiffInfo.key"
        output-format="side-by-side"/>
  </el-dialog>
</template>

<style scoped>

.version-option-tag {
  float: right;
  color: #909399FF;
  font-size: 13px;
}
</style>
