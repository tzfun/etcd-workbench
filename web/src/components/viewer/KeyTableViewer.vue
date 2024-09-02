<script setup lang="ts">
import {KeyDTO, KeyValueDTO} from "~/common/Types";
import {Delete, DocumentCopy, Edit, Finished, Search} from "@element-plus/icons-vue";
import {reactive} from "vue";
import {_isEmpty} from "~/common/Util";

const props = defineProps({
  data: {
    type: Array<KeyDTO>,
    required: true
  },
  hasMoreData: {
    type: Boolean
  }
})
const emits = defineEmits(['on-edit', 'on-diff', 'on-delete', 'copy-and-save', 'load-more'])

const keySearch = ref()
const selectedKey = ref<string[]>([])
const pagination = reactive({
  total: 0,
  pageSize: 15,
  page: 1
})
const filterTableData = computed(() => {
  if (_isEmpty(keySearch.value)) {
    pagination.total = props.data?.length
    let idx = (pagination.page - 1) * pagination.pageSize
    if (idx >= pagination.total) {
      pagination.page = 1;
      idx = 0;
    }
    return props.data?.slice(idx, idx + pagination.pageSize)
  } else {
    return props.data?.filter(
        (data) =>
            !keySearch.value ||
            data.key.toLowerCase().includes(keySearch.value.toLowerCase())
    )
  }
})

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

const deleteKey = (key:string) => {
  let idx = -1
  for (let i = 0; i <props.data!.length; i++) {
    if (props.data![i].key == key) {
      idx = i
      break
    }
  }

  if (idx >= 0) {
    props.data!.splice(idx, 1)
  }
}

const getSelectedKeys = (): string[] => {
  return selectedKey.value
}

const clearSelectedKeys = () => {
  selectedKey.value = []
}

const handlePaginationSizeChange = (val: number) => {
  pagination.pageSize = val
}

const handlePaginationPageChange = (val: number) => {
  pagination.page = val
}

defineExpose({
  getSelectedKeys,
  clearSelectedKeys,
  deleteKey
})

</script>

<template>
  <div style="padding-bottom: 15px;">
    <el-table :data="filterTableData"
              border
              stripe
              @selection-change="handleSelectionChange">
      <el-table-column type="selection" width="55"/>
      <el-table-column prop="key" label="Key" sortable min-width="230"/>
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
          <el-button type="warning" :icon="Finished" size="small" @click="copyAndSave(scope.row)">Copy And Save
          </el-button>
          <el-button type="danger" :icon="Delete" size="small" @click="del(scope.$index,scope.row)">Delete</el-button>
        </template>
      </el-table-column>
    </el-table>

    <div class="pagination-block" v-if="_isEmpty(keySearch)">
      <el-pagination
          v-model:current-page="pagination.page"
          v-model:page-size="pagination.pageSize"
          :page-sizes="[15, 50, 100, 300, 500]"
          layout="sizes, prev, pager, next"
          :total="pagination.total"
          @size-change="handlePaginationSizeChange"
          @current-change="handlePaginationPageChange"
      />
      <el-button type="primary"
                 v-show="hasMoreData"
                 @click="emits('load-more')"
      >Load More</el-button>
    </div>
  </div>
</template>

<style scoped>
.pagination-block {
  margin-top: 10px;
  display: flex;
  justify-content: center;
}
</style>
