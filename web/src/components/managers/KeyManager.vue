<script setup lang="ts">
import {deleteKey, getAllKeys, getKV, putKV} from "~/services/SessionService";
import {Delete, DocumentAdd, DocumentCopy, Edit, Refresh} from "@element-plus/icons-vue";
import {EditorConfig, KeyDTO, KeyValueDTO} from "~/entitys/TransformTypes";
import {ElMessageBox} from "element-plus";
import Editor from "~/components/editor/Editor.vue";
import {isDark} from "~/composables";
import {reactive} from "vue";
import {_isEmpty} from "~/util/BaseUtil";

const editorRef = ref(null)
const props = defineProps({
  sessionKey: String
})

onMounted(() => {
  loadAllKeys()
})

const tableData = ref<Array<KeyDTO>>([])
const editing = ref<Boolean>(false)
const isNew = ref<Boolean>(false)
const editingKV = ref<KeyValueDTO>()

const editorConfig = reactive<EditorConfig>({
  disabled: false,
  indentWithTab: true,
  tabSize: 2,
  autofocus: true,
  height: "500px",
  language: 'json',
  theme: isDark ? 'oneDark' : 'default'
})
const editorLanguage = computed(() => {
})

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

}

const del = (index, row: KeyDTO) => {
  ElMessageBox.confirm(
      'Are you sure to delete this key?',
      'Confirm',
      {
        confirmButtonText: 'OK',
        cancelButtonText: 'Cancel',
        type: 'warning',
      }
  ).then(() => {
    deleteKey(props.sessionKey, row.key).then(() => {
      ElMessage({
        type: 'success',
        message: 'Delete completed',
      })
      tableData.value.splice(index, 1)
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

  putKV(props.sessionKey, key, value).then(() => {
    loadAllKeys()
    editing.value = false
  }).catch(e => {
    console.error(e)
  })
}
</script>

<template>
  <div>
    <el-button type="primary" :icon="Refresh" @click="loadAllKeys">Refresh</el-button>
    <el-button type="success" :icon="DocumentAdd" @click="add">Add Key/Value</el-button>
  </div>

  <el-table :data="tableData" border stripe class="table">
    <el-table-column prop="key" label="Key" width="180"/>
    <el-table-column prop="version" label="Version" width="180"/>
    <el-table-column prop="createRevision" label="Create Revision"/>
    <el-table-column prop="modRevision" label="Modify Revision"/>
    <el-table-column prop="lease" label="Lease"/>
    <el-table-column fixed="right" label="Operations" width="300">
      <template #default="scope">
        <el-button type="primary" :icon="Edit" plain size="small" @click="edit(scope.$index,scope.row)">Edit</el-button>
        <el-button type="info" :icon="DocumentCopy" plain size="small" @click="diff(scope.$index,scope.row)">Version
          Diff
        </el-button>
        <el-button type="danger" :icon="Delete" size="small" @click="del(scope.$index,scope.row)">Delete</el-button>
      </template>
    </el-table-column>
  </el-table>

  <el-dialog v-model="editing" title="Key Editor" align-center>
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
            :config="editorConfig"
            :language="editorLanguage"/>
    <template #footer>
      <span class="dialog-footer">
        <el-button @click="editing = false">Cancel</el-button>
        <el-button type="primary" @click="putKey">
          Confirm
        </el-button>
      </span>
    </template>
  </el-dialog>
</template>

<style scoped>
.table {
  width: 100%;
  margin: 15px 0;
}
</style>
