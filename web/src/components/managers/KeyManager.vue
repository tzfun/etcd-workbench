<script setup lang="ts">
import {getAllKeys} from "~/services/SessionService";
import {Delete, DocumentAdd, DocumentCopy, Edit, Refresh} from "@element-plus/icons-vue";
import {KeyDTO} from "~/entitys/TransformTypes";

const props = defineProps({
  sessionKey: String
})

onMounted(() => {
  getAllKeys(props.sessionKey as string).then(data => {
    console.debug(data)
    tableData.value = data
  })
})

const tableData = ref<Array<KeyDTO>>([])

const edit = (index, row: KeyDTO) => {
  console.log(index, row)
}

const diff = (index, row: KeyDTO) => {

}

const del = (index, row: KeyDTO) => {

}
</script>

<template>
  <div>
    <el-button :icon="Refresh">Refresh</el-button>
    <el-button :icon="DocumentAdd">Add Key/Value</el-button>
  </div>

  <el-table :data="tableData" border stripe class="table">
    <el-table-column prop="key" label="Key" width="180"/>
    <el-table-column prop="version" label="Version" width="180"/>
    <el-table-column prop="createRevision" label="Create Revision"/>
    <el-table-column prop="modRevision" label="Modify Revision"/>
    <el-table-column prop="lease" label="Lease"/>
    <el-table-column fixed="right" label="Operations">
      <template #default="scope">
        <el-button type="primary" :icon="Edit" plain size="small" @click="edit(scope.$index,scope.row)">Edit</el-button>
        <el-button type="info" :icon="DocumentCopy" plain size="small" @click="diff(scope.$index,scope.row)">Version
          Diff
        </el-button>
        <el-button type="danger" :icon="Delete" size="small" @click="del(scope.$index,scope.row)">Delete</el-button>
      </template>
    </el-table-column>
  </el-table>
</template>

<style scoped>
.table {
  width: 100%;
}
</style>