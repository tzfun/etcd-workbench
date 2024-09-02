<script setup lang="ts">
import {Delete, Lock, Refresh, Search, User} from "@element-plus/icons-vue";
import {
  _addRole,
  _deleteRole,
  _getRolePermission,
  _listRoles,
  _roleGrantPermission,
  _roleRevokePermission
} from "~/common/Service";
import {_isEmpty} from "~/common/Util";
import {FormInstance} from "element-plus";

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
  _listRoles(props.sessionKey).then(data => {
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
    _addRole(props.sessionKey, value).then(() => {
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
    _deleteRole(props.sessionKey, role).then(() => {
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
const grantPermissionFormRef = ref<FormInstance>()
const grantPermissionForm = reactive<Permission>({
  type: '',
  key: '',
  allKeys: false,
  prefix: false
})
const openPermissionDialog = (role: string, idx: number) => {
  _getRolePermission(props.sessionKey, role).then(data => {
    permissions.value = data
    curRole.value = role
    resetGrantPermissionForm()
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
    _roleRevokePermission(props.sessionKey, role, permission).then(() => {
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

const resetGrantPermissionForm = () => {
  grantPermissionForm.type = ''
  grantPermissionForm.key = ''
  grantPermissionForm.allKeys = false
  grantPermissionForm.prefix = false
}

const grantPermission = () => {
  if (_isEmpty(grantPermissionForm.type)) {
    ElMessage({
      type: 'info',
      message: 'No permission selected',
    })
    return
  }
  if (!grantPermissionForm.allKeys && _isEmpty(grantPermissionForm.key)) {
    ElMessage({
      type: 'info',
      message: 'Key cannot be empty',
    })
    return
  }
  const newPermission = grantPermissionForm as Permission
  _roleGrantPermission(props.sessionKey, curRole.value, newPermission).then(() => {
    ElMessage({
      type: 'success',
      message: 'Grant permission successfully',
    })
    permissions.value.push(JSON.parse(JSON.stringify(newPermission)))
  })
}

</script>

<template>
  <div class="page">
    <div class="mb-5 button-list">
      <el-button @click="loadAllRole" :icon="Refresh">Refresh</el-button>
      <el-button type="primary" :icon="User" @click="add">Add Role</el-button>
    </div>
    <el-table :data="filterTableData"
              border
              stripe
              class="mb-5">
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
               :title="`Role permissions: ${curRole}`"
               append-to-body
               align-center>

      <el-card class="mb-10">
        <el-form ref="grantPermissionFormRef"
                 :model="grantPermissionForm"
                 label-width="100px"
                 label-position="right"
                 label-suffix=":">
          <el-form-item label="Key Type">
            <el-checkbox v-model="grantPermissionForm.allKeys" label="ALL KEYS" prop="allKeys"/>
            <el-checkbox v-show="!grantPermissionForm.allKeys" v-model="grantPermissionForm.prefix" label="For Prefix"
                         prop="prefix"/>
          </el-form-item>
          <el-form-item label="Key" v-show="!grantPermissionForm.allKeys" prop="key">
            <el-input type="text" v-model="grantPermissionForm.key" placeholder="Input key"/>
          </el-form-item>
          <el-form-item label="Permission" prop="type">
            <el-select v-model="grantPermissionForm.type">
              <el-option value="READ" label="Read Only"></el-option>
              <el-option value="WRITE" label="Write Only"></el-option>
              <el-option value="READWRITE" label="Read And Write"></el-option>
            </el-select>
          </el-form-item>
          <el-form-item>
            <el-button type="primary" @click="grantPermission()">
              Add Permission
            </el-button>
            <el-button @click="resetGrantPermissionForm()">Reset</el-button>
          </el-form-item>
        </el-form>
      </el-card>


      <el-table :data="permissions"
                border
                stripe>
        <el-table-column label="Key">
          <template #default="{row}">
            <span v-if="row.allKeys" style="font-weight: 600;color: #ab99bb;">ALL KEYS</span>
            <span v-else>{{ row.key }}</span>
          </template>
        </el-table-column>
        <el-table-column label="Prefix">
          <template #default="{row}">
            {{ row.allKeys ? '' : row.prefix }}
          </template>
        </el-table-column>
        <el-table-column prop="type" label="Permission"/>

        <el-table-column label="Operations">
          <template #default="{row, $index}">
            <el-button type="danger"
                       size="small"
                       @click="revokePermission(curRole, row, $index)">Revoke
            </el-button>
          </template>
        </el-table-column>
      </el-table>

    </el-dialog>
  </div>
</template>

<style lang="scss" scoped>
@import '../../styles/index.scss';
</style>
