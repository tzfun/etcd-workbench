<script setup lang="ts">
import {addUser, deleteUser, listRoles, listUser, userGrantRole, userRevokeRole} from "~/services/SessionService";
import {Delete, Plus, Refresh, Search, UserFilled} from "@element-plus/icons-vue";
import {_isEmpty} from "~/util/BaseUtil";

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
const allRoles = ref([])

const openRoleDialog = (user: User, idx: number) => {
  listRoles(props.sessionKey).then(data => {
    allRoles.value = data
    curUser.value = user
    curUserIdx.value = idx
    showRoleDialog.value = true
  }).catch(e => {
    console.log(e)
  })
}

const grantUserRole = () => {
  if (_isEmpty(grantRole.value)) {
    ElMessage({
      type: 'warning',
      message: 'Role name can not be empty',
    })
    return
  }

  userGrantRole(props.sessionKey, curUser.value?.user, grantRole.value).then(() => {
    tableData.value[curUserIdx].roles.push(grantRole.value)
    curUser.value?.roles.push(grantRole.value)
  }).catch(e => {
    console.error(e)
  })
}

const revokeRole = (user: string, role: string, idx: number) => {
  ElMessageBox.confirm(
      `Are you sure to revoke this role? ${user} :<br><strong> ${role}</strong>`,
      'Confirm',
      {
        confirmButtonText: 'OK',
        cancelButtonText: 'Cancel',
        dangerouslyUseHTMLString: true,
        type: 'warning',
      }
  ).then(() => {
    userRevokeRole(props.sessionKey, user, role).then(() => {
      tableData.value[curUserIdx].roles.splice(grantRole.value, 1)
      curUser.value?.roles.splice(idx, 1)
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

  addUser(props.sessionKey, addForm.user, addForm.password).then(() => {
    showAddDialog.value = false
    loadAllUser()
  }).catch(e => {
    console.error(e)
  })
}
</script>

<template>
  <div>
    <el-row>
      <el-button @click="loadAllUser" :icon="Refresh">Refresh Table</el-button>
      <el-button type="primary" :icon="UserFilled" @click="openAddDialog">Add User</el-button>
    </el-row>
    <el-table :data="filterTableData"
              border
              stripe>
      <el-table-column prop="user" label="User" sortable/>
      <el-table-column label="Roles">
        <template #default="{row}">
          {{ row.roles == null ? '' : row.roles.join(",") }}
        </template>
      </el-table-column>
      <el-table-column label="Operations">
        <template #header>
          <el-input v-model="keySearch" placeholder="Type to search" :prefix-icon="Search"/>
        </template>
        <template #default="{row, $index}">
          <el-button type="primary" :icon="UserFilled" size="small" @click="openRoleDialog(row, $index)">Edit Roles
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
               width="500"
               align-center>
      <el-row>
        <el-input v-model="grantRole" placeholder="Input grant role name"></el-input>
        <el-button type="primary" :icon="Plus" @click="grantUserRole()">Grant Role</el-button>
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
            <el-button type="danger" size="small" @click="revokeRole(curUser.user, row, $index)"> revoke</el-button>
          </template>
        </el-table-column>
      </el-table>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="showAddDialog = false">Cancel</el-button>
          <el-button type="primary" @click="add">Confirm</el-button>
        </span>
      </template>
    </el-dialog>
  </div>

</template>

<style scoped>

</style>