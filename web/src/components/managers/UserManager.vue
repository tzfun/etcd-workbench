<script setup lang="ts">
import {
  addUser, authDisable,
  authEnable,
  deleteUser,
  listRoles,
  listUser,
  userGrantRole,
  userRevokeRole
} from "~/common/Service";
import {Delete, Lock, Plus, Refresh, Search, Unlock, UserFilled} from "@element-plus/icons-vue";
import {_isEmpty} from "~/common/Util";
import {ElMessage} from "element-plus";

const props = defineProps({
  sessionKey: String
})

type User = {
  user: string,
  roles: string[]
}

onMounted(() => {
  loadAllUser()
})

const tableData = ref([])
const filterTableData = computed(() =>
    tableData.value.filter(
        (data) =>
            !keySearch.value ||
            data.user.toLowerCase().includes(keySearch.value.toLowerCase())
    )
)
const keySearch = ref()

const loadAllUser = () => {
  listUser(props.sessionKey).then(data => {
    tableData.value = data
  }).catch(e => {
    console.error(e)
  })
}

const del = (user: string, idx: number) => {
  ElMessageBox.confirm(
      `Are you sure to delete this user? <br><strong>${user}</strong>`,
      'Confirm',
      {
        confirmButtonText: 'OK',
        cancelButtonText: 'Cancel',
        dangerouslyUseHTMLString: true,
        type: 'warning',
      }
  ).then(() => {
    deleteUser(props.sessionKey, user).then(() => {
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

const showRoleDialog = ref(false)
const curUser = ref<User>()
const curUserIdx = ref()
const grantRole = ref()
const allRoles = ref(null)

const openRoleDialog = async (user: User, idx: number) => {
  if (allRoles.value == null) {
    await loadAllRoles()
  }
  curUser.value = user
  curUserIdx.value = idx
  showRoleDialog.value = true
}

const loadAllRoles = async () => {
  allRoles.value = await listRoles(props.sessionKey)
}

const grantUserRole = () => {
  if (_isEmpty(grantRole.value)) {
    ElMessage({
      type: 'warning',
      message: 'No role selected',
    })
    return
  }
  if (tableData.value[curUserIdx.value].roles.includes(grantRole.value)) {
    ElMessage({
      type: 'info',
      message: 'Role already exists',
    })
    return
  }
  userGrantRole(props.sessionKey, curUser.value?.user, grantRole.value).then(() => {
    tableData.value[curUserIdx.value].roles.push(grantRole.value)
  }).catch(e => {
    console.error(e)
  })
}

const revokeRole = (user: string, role: string, idx: number) => {
  ElMessageBox.confirm(
      `Are you sure to revoke this role? ${user} :<strong> ${role}</strong>`,
      'Confirm',
      {
        confirmButtonText: 'OK',
        cancelButtonText: 'Cancel',
        dangerouslyUseHTMLString: true,
        type: 'warning',
      }
  ).then(() => {
    userRevokeRole(props.sessionKey, user, role).then(() => {
      tableData.value[curUserIdx.value].roles.splice(grantRole.value, 1)
    }).catch(e => {
      console.error(e)
    })
  }).catch(() => {
  })
}

const showAddDialog = ref(false)
const addForm = reactive({
  user: '',
  password: ''
})

const openAddDialog = () => {
  addForm.user = ''
  addForm.password = ''
  showAddDialog.value = true
}

const add = () => {
  if (_isEmpty(addForm.user)) {
    ElMessage({
      type: 'warning',
      message: 'User cannot be empty',
    })
    return
  }

  if (_isEmpty(addForm.password)) {
    ElMessage({
      type: 'warning',
      message: 'Password cannot be empty',
    })
    return
  }

  addUser(props.sessionKey!, addForm.user, addForm.password).then(() => {
    showAddDialog.value = false
    loadAllUser()
  }).catch(e => {
    console.error(e)
  })
}

const doAuthEnable = () => {
  authEnable(props.sessionKey!).then(() => {
    ElMessage({
      type: 'success',
      message: 'Enabled authentication',
    })
  }).catch(e => {
    console.error(e)
  })
}

const doAuthDisable = () => {
  authDisable(props.sessionKey!).then(() => {
    ElMessage({
      type: 'success',
      message: 'Disabled authentication',
    })
  }).catch(e => {
    console.error(e)
  })
}
</script>

<template>
  <div class="page">
    <div class="mb-5">
      <el-button @click="loadAllUser" :icon="Refresh">Refresh Table</el-button>
      <el-button type="primary" :icon="UserFilled" @click="openAddDialog">Add User</el-button>
      <el-button type="success" :icon="Lock" @click="doAuthEnable">Auth Enable</el-button>
      <el-button type="danger" :icon="Unlock" @click="doAuthDisable">Auth Disable</el-button>
    </div>
    <el-table :data="filterTableData"
              border
              stripe
              class="mb-5">
      <el-table-column prop="user" label="User" sortable/>
      <el-table-column label="Roles">
        <template #default="{row}">
          {{ row.roles == null ? '' : row.roles.join(", ") }}
        </template>
      </el-table-column>
      <el-table-column label="Operations">
        <template #header>
          <el-input v-model="keySearch" placeholder="Type to search" :prefix-icon="Search"/>
        </template>
        <template #default="{row, $index}">
          <el-button type="primary" :icon="UserFilled" size="small" @click="openRoleDialog(row, $index)">Roles
          </el-button>
          <el-button type="danger" :icon="Delete" size="small" @click="del(row.user, $index)">Delete</el-button>
        </template>
      </el-table-column>
    </el-table>

    <el-dialog v-model="showAddDialog"
               title="Add user"
               :close-on-click-modal="false"
               width="500"
               align-center>
      <el-form :model="addForm" label-position="left" label-suffix=":" label-width="100px">
        <el-form-item label="User">
          <el-input type="text" v-model="addForm.user" placeholder="Input user login account"></el-input>
        </el-form-item>
        <el-form-item label="Password">
          <el-input type="password" v-model="addForm.password" show-password
                    placeholder="Input user login password"></el-input>
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="showAddDialog = false">Cancel</el-button>
          <el-button type="primary" @click="add">Confirm</el-button>
        </span>
      </template>
    </el-dialog>

    <el-dialog v-model="showRoleDialog"
               title="User roles"
               width="700"
               align-center>
      <el-row class="mt-4 mb-4">
        <el-col :span="12">
          <el-select v-model="grantRole" placeholder="Select role" style="width: 100%">
            <el-option v-for="role in allRoles " :key="role" :value="role" :label="role"></el-option>
          </el-select>
        </el-col>
        <el-col :span="11" :offset="1">
          <el-button-group>
            <el-button type="primary"
                       style="padding: 8px 15px;"
                       :icon="Plus"
                       @click="grantUserRole">Grant Role</el-button>
            <el-button :icon="Refresh"
                       style="padding: 8px 15px;"
                       @click="loadAllRoles"></el-button>
          </el-button-group>
        </el-col>
      </el-row>
      <el-table :data="curUser.roles"
                border
                stripe>
        <el-table-column label="Role">
          <template #default="{row}">
            {{ row }}
          </template>
        </el-table-column>

        <el-table-column label="Operations">
          <template #default="{row, $index}">
            <el-button type="danger"
                       size="small"
                       @click="revokeRole(curUser.user, row, $index)">Revoke</el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-dialog>
  </div>

</template>

<style lang="scss" scoped>
@import '../../styles/index.scss';
</style>
