<script setup lang="ts">
import {KeyDTO, KeyValueDTO} from "~/entitys/TransformTypes";
import {Delete, DocumentCopy, Edit, Finished, Search} from "@element-plus/icons-vue";
import Editor from "~/components/editor/Editor.vue";

const props = defineProps({
  data: Array<KeyDTO>
})
const emits = defineEmits(['on-edit', 'on-diff', 'on-delete', 'copy-and-save'])

const keySearch = ref()
const selectedKey = ref<string[]>([])
const filterTableData = computed(() =>
    props.data?.filter(
        (data) =>
            !keySearch.value ||
            data.key.toLowerCase().includes(keySearch.value.toLowerCase())
    )
)

const handleSelectionChange = (rows: KeyValueDTO[]) => {
  let selected = []
  for (let row of rows) {
    selected.push(row.key)
  }
  selectedKey.value = selected
}

const edit = (row: KeyDTO) => {
  emits('on-edit', row)
}

const diff = (row: KeyDTO) => {
  emits('on-diff', row)
}

const copyAndSave = (row: KeyDTO) => {
  emits('copy-and-save', row.key)
}

const del = (index: number, row: KeyDTO) => {
  emits('on-delete', {
    key: row.key,
    callback: () => {
      props.data!.splice(index, 1)
    }
  })
}

const getSelectedKeys = ():string[] => {
  return selectedKey.value
}

const clearSelectedKeys = () => {
  selectedKey.value = []
}

defineExpose({
  getSelectedKeys,
  clearSelectedKeys
})

</script>

<template>
  <div style="padding-bottom: 15px;">
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
      <el-table-column fixed="right" label="Operations" width="430">
        <template #header>
          <el-input v-model="keySearch" placeholder="Type to search" :prefix-icon="Search"/>
        </template>
        <template #default="scope">
          <el-button type="primary" :icon="Edit" size="small" @click="edit(scope.row)">Edit
          </el-button>
          <el-button type="info" :icon="DocumentCopy" size="small" @click="diff(scope.row)">Version
            Diff
          </el-button>
          <el-button type="warning" :icon="Finished" size="small" @click="copyAndSave(scope.row)">Copy And Save</el-button>
          <el-button type="danger" :icon="Delete" size="small" @click="del(scope.$index,scope.row)">Delete</el-button>
        </template>
      </el-table-column>
    </el-table>
  </div>
</template>

<style scoped>

</style>
