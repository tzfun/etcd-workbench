<script setup lang="ts">
import {Delete, Lock, Refresh, Search, User} from "@element-plus/icons-vue";
import {addRole, deleteRole, getRolePermission, listRoles, roleRevokePermission} from "~/services/SessionService";
import {_isEmpty} from "~/util/BaseUtil";

const props = defineProps({
  sessionKey: String
})

onMounted(() => {
  loadAllRole()
})

const tableData = ref([])
const filterTableData = computed(() =>
    tableData.value.filter(
        (data) =>
            !keySearch.value ||
            data.role.toLowerCase().includes(keySearch.value.toLowerCase())
    )
)
const keySearch = ref()

const loadAllRole = () => {
  listRoles(props.sessionKey).then(data => {
    tableData.value = data
  }).catch(e => {
    console.error(e)
  })
}

const add = () => {
  ElMessageBox.prompt('Please input role name', 'Tip', {
    confirmButtonText: 'OK',
    cancelButtonText: 'Cancel',
  }).then(({value}) => {
    if (_isEmpty(value)) {
      ElMessage({
        type: 'warning',
        message: 'Role name cannot be empty',
      })
      return
    }
    addRole(props.sessionKey, value).then(() => {
      tableData.value.push(value)
    })
  }).catch(() => {
  })
}

const del = (role: string, idx: number) => {
  ElMessageBox.confirm(
      `Are you sure to delete this role? <br><strong>${role}</strong>`,
      'Confirm',
      {
        confirmButtonText: 'OK',
        cancelButtonText: 'Cancel',
        dangerouslyUseHTMLString: true,
        type: 'warning',
      }
  ).then(() => {
    deleteRole(props.sessionKey, role).then(() => {
      ElMessage({
        type: 'success',
        message: 'Deleted successfully',
      })
      tableData.value.splice(idx, 1)
    }).catch(e => {
      console.error(e)
    })
  }).catch(() => {
  })
}

type Permission = {
  type: string,
  key: string,
  allKeys: boolean,
  prefix: boolean
}
const showPermissionDialog = ref(false)
const curRole = ref()
const permissions = ref<Array<Permission>>([])
const viewPermissionList = ref(true)
const grantPermissionForm = reactive<Permission>({
  type: '',
  key: '',
  allKeys: false,
  prefix: false
})
const openPermissionDialog = (role: string, idx: number) => {
  getRolePermission(props.sessionKey, role).then(data => {
    permissions.value = data
    curRole.value = role
    grantPermissionForm.type = ''
    grantPermissionForm.key = ''
    grantPermissionForm.allKeys = false
    grantPermissionForm.prefix = false
    showPermissionDialog.value = true
  })
}

const revokePermission = (role: string, permission: Permission, idx: number) => {
  ElMessageBox.confirm(
      `Are you sure to revoke ${role}'s permission?`,
      'Confirm',
      {
        confirmButtonText: 'OK',
        cancelButtonText: 'Cancel',
        dangerouslyUseHTMLString: true,
        type: 'warning',
      }
  ).then(() => {
    roleRevokePermission(props.sessionKey, role, permission).then(() => {
      ElMessage({
        type: 'success',
        message: 'Revoked successfully',
      })
      permissions.value.splice(idx, 1)
    }).catch(e => {
      console.error(e)
    })
  }).catch(() => {
  })
}

const switchPermissionTag = () => {
  viewPermissionList.value = !viewPermissionList.value
}
</script>

<template>
  <div>
    <el-row>
      <el-button @click="loadAllRole" :icon="Refresh">Refresh Table</el-button>
      <el-button type="primary" :icon="User" @click="add">Add Role</el-button>
    </el-row>
    <el-table :data="filterTableData"
              border
              stripe>
      <el-table-column label="Role" sortable>
        <template #default="{row}">
          {{ row }}
        </template>
      </el-table-column>
      <el-table-column label="Operations">
        <template #header>
          <el-input v-model="keySearch" placeholder="Type to search" :prefix-icon="Search"/>
        </template>
        <template #default="{row, $index}">
          <el-button type="primary"
                     :icon="Lock"
                     size="small"
                     @click="openPermissionDialog(row, $index)">
            Permissions
          </el-button>
          <el-button type="danger"
                     :icon="Delete"
                     size="small"
                     @click="del(row, $index)">
            Delete
          </el-button>
        </template>
      </el-table-column>
    </el-table>

    <el-dialog v-model="showPermissionDialog"
               title="Role permissions"
               align-center>
      <el-button type="primary" @click="switchPermissionTag">Grant Permission</el-button>
      <el-table :data="permissions"
                v-show="viewPermissionList"
                border
                stripe>
        <el-table-column label="Key">
          <template #default="{row}">
            <span v-if="row.allKeys">ALL KEYS</span>
            <span v-else>{{ row.key }}</span>
          </template>
        </el-table-column>
        <el-table-column prop="prefix" label="Prefix"/>
        <el-table-column prop="type" label="Permission"/>

        <el-table-column label="Operations">
          <template #default="{row, $index}">
            <el-button type="danger"
                       size="small"
                       @click="revokePermission(curRole, row, $index)">Revoke</el-button>
          </template>
        </el-table-column>
      </el-table>
      <el-form v-show="!viewPermissionList"
               :model="grantPermissionForm"
               label-width="100px"
               label-position="left"
               label-suffix=":">
        <el-form-item label="Key Type">
          <el-radio v-model="grantPermissionForm.allKeys">ALL KEYS</el-radio>
        </el-form-item>
        <el-form-item lable="Key" v-show="!grantPermissionForm.allKeys">
          <el-input type="text" v-model="grantPermissionForm.key" placeholder="Input key"/>
        </el-form-item>
        <el-form-item label="Permission">
          <el-select v-model="grantPermissionForm.type">
            <el-option value="READ" label="Read Only"></el-option>
            <el-option value="WRITE" label="Write Only"></el-option>
            <el-option value="READWRITE" label="Read And Write"></el-option>
            <el-option value="UNRECOGNIZED" label="Unrecognized"></el-option>
          </el-select>
        </el-form-item>
      </el-form>
    </el-dialog>
  </div>
</template>

<style scoped>

</style>