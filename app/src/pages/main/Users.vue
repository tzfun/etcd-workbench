<script setup lang="ts">
import {onMounted, PropType, reactive, ref} from "vue";
import {SessionData} from "~/common/transport/connection.ts";
import {
  _addUser,
  _authDisable,
  _authEnable,
  _deleteUser,
  _getAllRoles,
  _getAllUsers,
  _handleError,
  _userChangePassword,
  _userGrantRole,
  _userRevokeRole
} from "~/common/services.ts";
import {User} from "~/common/transport/user.ts";
import {_confirmSystem, _emitLocal, _tipWarn, EventName} from "~/common/events.ts";
import {_isEmpty, _shuffleArray} from "~/common/utils.ts";

const colorList = [
  'red',
  'pink',
  'purple',
  'deep-purple',
  'indigo',
  'blue',
  'light-blue',
  'cyan',
  'teal',
  'green',
  'light-green',
  'lime',
  'amber',
  'orange',
  'deep-orange',
  'brown',
  'blue-grey',
  'grey'
]

const props = defineProps({
  session: {
    type: Object as PropType<SessionData>,
    required: true
  }
})

const users = ref<User[]>([])
const roleColors = ref<Record<string, string>>({})
const roleColorIdx = ref<number>(0)
const editUserDialog = reactive({
  show: false,
  newUser: false,
  title: '',
  user: '',
  password: '',
})
const roles = ref<string[]>()

const loadingStore = reactive({
  loadAllUser: false,
  addUser: false,
  authEnable: false,
  authDisable: false,
  editUser: false,
  grantRole: false
})

const search = ref('')
const grantRoleDialog = reactive({
  show: false,
  role: '',
  user: '',
  userIdx: 0
})

onMounted(() => {
  loadAllUser()
})

const loadAllUser = () => {
  roles.value = undefined
  loadingStore.loadAllUser = true
  roleColorIdx.value = 0
  _shuffleArray(colorList)
  _getAllUsers(props.session?.id).then(data => {
    let colorIdx = 0;
    let roleColorMap: Record<string, string> = {}
    for (let user of data) {
      let root = user.user == 'root'

      for (let role of user.roles) {
        if (role == 'root') {
          root = true
        }
        if (roleColorMap[role]) {
          continue
        }
        roleColorMap[role] = colorList[colorIdx++]
        if (colorIdx == colorList.length) {
          colorIdx = 0
        }
      }
      roleColorIdx.value = colorIdx
      user.root = root
    }
    roleColors.value = roleColorMap
    users.value = data
  }).catch(e => {
    _handleError({
      e,
      session: props.session
    })
  }).finally(() => {
    loadingStore.loadAllUser = false
  })
}

const authEnable = () => {
  _confirmSystem('Are you sure you want to turn on the authentication function? You will need to reconnect after executing.').then(() => {
    loadingStore.authEnable = true
    _authEnable(props.session?.id).then(() => {
      _emitLocal(EventName.CLOSE_TAB, props.session!.id)
    }).catch(e => {
      _handleError({
        e,
        session: props.session
      })
    }).finally(() => {
      loadingStore.authEnable = false
    })
  })
}

const authDisable = () => {
  _confirmSystem('Are you sure you want to turn off authentication? You will need to reconnect after executing this command.').then(() => {
    loadingStore.authDisable = true
    _authDisable(props.session?.id).then(() => {
      _emitLocal(EventName.CLOSE_TAB, props.session!.id)
    }).catch(e => {
      _handleError({
        e,
        session: props.session
      })
    }).finally(() => {
      loadingStore.authDisable = false
    })
  })
}

const revokeUserRole = (user: string, role: string, userIdx: number, roleIdx: number) => {
  _confirmSystem(`Confirm to revoke role <strong>${role}</strong> of user <strong>${user}</strong>?`).then(() => {
    _userRevokeRole(props.session?.id, user, role).then(() => {
      users.value[userIdx].roles.splice(roleIdx, 1)
    }).catch(e => {
      _handleError({
        e,
        session: props.session
      })
    })
  }).catch(() => {
  })
}

const openChangePasswordDialog = (user: string) => {
  editUserDialog.password = ''
  editUserDialog.user = user
  editUserDialog.title = 'Change Password'
  editUserDialog.newUser = false
  editUserDialog.show = true
}

const changePassword = () => {
  if (_isEmpty(editUserDialog.password)) {
    _tipWarn("New password can not be empty")
    return
  }
  loadingStore.editUser = true
  _userChangePassword(props.session?.id, editUserDialog.user, editUserDialog.password).then(() => {
    if (props.session.user == editUserDialog.user) {
      _emitLocal(EventName.CLOSE_TAB, props.session!.id)
    }
    editUserDialog.show = false
  }).catch(e => {
    _handleError({
      e,
      session: props.session
    })
  }).finally(() => {
    loadingStore.editUser = false
  })
}

const openNewUserDialog = () => {
  editUserDialog.password = ''
  editUserDialog.user = ''
  editUserDialog.title = 'New User'
  editUserDialog.newUser = true
  editUserDialog.show = true
}

const newUser = () => {
  if (_isEmpty(editUserDialog.password)) {
    _tipWarn("New password can not be empty")
    return
  }
  loadingStore.editUser = true
  _addUser(props.session?.id, editUserDialog.user, editUserDialog.password).then(() => {
    users.value.push({
      user: editUserDialog.user,
      roles: [],
      root: false
    })
    editUserDialog.show = false
  }).catch(e => {
    _handleError({
      e,
      session: props.session
    })
  }).finally(() => {
    loadingStore.editUser = false
  })
}

const confirmEditUser = () => {
  if (editUserDialog.newUser) {
    newUser()
  } else {
    changePassword()
  }
}

const deleteUser = (user: string, userIdx: number) => {
  _confirmSystem(`Are you sure you want to delete this user?<br/><strong>${user}</strong>`).then(() => {
    _deleteUser(props.session?.id, user).then(() => {
      users.value.splice(userIdx, 1)
    })
  }).catch(() => {
  })
}

const openGrantRoleDialog = (user: string, userIdx: number) => {
  grantRoleDialog.role = ''
  grantRoleDialog.user = user
  grantRoleDialog.userIdx = userIdx

  if (roles.value) {
    grantRoleDialog.show = true
  } else {
    _getAllRoles(props.session?.id).then(data => {
      roles.value = data
      grantRoleDialog.show = true
    }).catch(e => {
      _handleError({
        e,
        session: props.session
      })
    })
  }
}

const grantRole = () => {
  if (_isEmpty(grantRoleDialog.role)) {
    _tipWarn("Please select a role")
    return
  }
  loadingStore.grantRole = true
  let user = grantRoleDialog.user
  let role = grantRoleDialog.role
  _userGrantRole(props.session?.id, user, role).then(() => {
    if (!roleColors.value[role]) {
      roleColors.value[role] = colorList[roleColorIdx.value++]
      if (roleColorIdx.value == colorList.length) {
        roleColorIdx.value = 0
      }
    }
    users.value[grantRoleDialog.userIdx].roles.push(role)
    grantRoleDialog.show = false
  }).catch(e => {
    _handleError({
      e,
      session: props.session
    })
  }).finally(() => {
    loadingStore.grantRole = false
  })
}

const roleSelectionProps = (item: string) => {
  return {
    title: item,
    value: item,
    disabled: users.value[grantRoleDialog.userIdx].roles.includes(item)
  }
}

</script>

<template>
  <div class="fill-height pa-5 overflow-y-auto">
    <div>
      <v-btn 
            v-bind="props"
            variant="tonal"
            size="small"
            icon="mdi-refresh"
            @click="loadAllUser"
            :loading="loadingStore.loadAllUser"
            title="Refresh"
      ></v-btn>
      <v-btn class="text-none ml-2"
             prepend-icon="mdi-account-plus-outline"
             @click="openNewUserDialog"
             color="green"
             text="Add User"
      ></v-btn>
      <v-btn class="text-none ml-2"
             prepend-icon="mdi-lock"
             @click="authEnable"
             color="yellow"
             text="Auth Enable"
             :loading="loadingStore.authEnable"
      ></v-btn>
      <v-btn class="text-none ml-2"
             prepend-icon="mdi-lock-open-variant"
             @click="authDisable"
             color="red"
             text="Auth Disable"
             :loading="loadingStore.authDisable"
      ></v-btn>
    </div>
    <div>
      <v-card class="mt-5 mb-5 overflow-x-auto"
              border
      >
        <v-card-text>
          <v-table hover style="min-width: 840px">
            <thead>
            <tr>
              <th class="text-left user-col font-weight-bold">
                User
              </th>
              <th class="text-left role-col font-weight-bold">
                Roles
              </th>
              <th class="text-left op-col">
                <v-text-field
                    v-model="search"
                    density="compact"
                    label="Search"
                    prepend-inner-icon="mdi-magnify"
                    variant="solo-filled"
                    flat
                    hide-details
                    single-line
                ></v-text-field>
              </th>
            </tr>
            </thead>
            <tbody>

            <tr
                v-for="(user, idx) in users"
                :key="idx"
                v-show="_isEmpty(search) || user.user.toLocaleString().includes(search.toLocaleString()) || user.roles.join(',').toLowerCase().includes(search.toLowerCase())"
            >
              <td class="user-col">{{ user.user }}</td>
              <td class="role-col">
                <v-chip v-for="(role, i) in user.roles"
                        :key="i"
                        label
                        class="mr-2 user-role-tag"
                        density="comfortable"
                        :text="role"
                        :color="roleColors[role]"
                >
                  <template v-slot:append>
                    <v-icon class="role-tag-close-icon"
                            @click="revokeUserRole(user.user, role, idx, i)"
                            color="secondary"
                    >mdi-close-circle
                    </v-icon>
                  </template>
                </v-chip>
              </td>
              <td class="op-col">
                <v-btn text="Grant Role"
                       color="green"
                       class="text-none"
                       size="small"
                       prepend-icon="mdi-account-details-outline"
                       @click="openGrantRoleDialog(user.user,idx)"
                ></v-btn>
                <v-btn text="Change Password"
                       color="yellow"
                       class="text-none ml-2"
                       size="small"
                       prepend-icon="mdi-lock-reset"
                       @click="openChangePasswordDialog(user.user)"
                ></v-btn>
                <v-btn text="Delete"
                       v-if="user.user != session.user"
                       prepend-icon="mdi-file-document-minus-outline"
                       color="red"
                       class="text-none ml-2"
                       size="small"
                       @click="deleteUser(user.user, idx)"
                ></v-btn>
              </td>
            </tr>
            </tbody>
          </v-table>
        </v-card-text>
      </v-card>
    </div>

    <!--  新增用户和修改密码弹窗 -->
    <v-dialog
        v-model="editUserDialog.show"
        persistent
        max-width="500px"
        min-width="200px"
        scrollable
    >
      <v-card :title="editUserDialog.title">
        <v-card-text>
          <v-layout class="mb-5">
            <span class="inline-label input-label">User: </span>
            <v-text-field v-model="editUserDialog.user"
                          density="comfortable"
                          prepend-inner-icon="mdi-account"
                          hide-details
                          placeholder="Please input user account"
                          :readonly="!editUserDialog.newUser"
                          width="400px"
            ></v-text-field>
          </v-layout>
          <v-layout class="mb-5">
            <span class="inline-label input-label">New Password: </span>
            <v-text-field v-model="editUserDialog.password"
                          type="password"
                          density="comfortable"
                          prepend-inner-icon="mdi-lock-reset"
                          placeholder="Please input user password"
                          hide-details
                          width="400px"
            ></v-text-field>
          </v-layout>
        </v-card-text>
        <v-card-actions>
          <v-btn text="Cancel"
                 variant="text"
                 class="text-none"
                 @click="editUserDialog.show = false"
          ></v-btn>

          <v-btn text="Commit"
                 variant="flat"
                 class="text-none"
                 color="primary"
                 @click="confirmEditUser"
                 :loading="loadingStore.editUser"
          ></v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>


    <!--  授予角色弹窗 -->
    <v-dialog
        v-model="grantRoleDialog.show"
        persistent
        max-width="500px"
        min-width="200px"
        scrollable
    >
      <v-card title="Grant Role">
        <v-card-text>
          <v-select :items="roles"
                    :item-props="roleSelectionProps"
                    v-model="grantRoleDialog.role"
                    placeholder="Role"
                    hint="Please select a role"
                    persistent-hint
                    density="comfortable"
          >
          </v-select>
        </v-card-text>
        <v-card-actions>
          <v-btn text="Cancel"
                 variant="text"
                 class="text-none"
                 @click="grantRoleDialog.show = false"
          ></v-btn>

          <v-btn text="Commit"
                 variant="flat"
                 class="text-none"
                 color="primary"
                 @click="grantRole"
                 :loading="loadingStore.grantRole"
          ></v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>

<style scoped lang="scss">

.user-col {
  min-width: 200px;
}

.role-col {
  min-width: 200px;
}

.op-col {
  min-width: 300px;
}

.role-tag-close-icon {
  font-size: 0;
  transition: all ease 0.15s;
}

.user-role-tag:hover {
  .role-tag-close-icon {
    font-size: 1rem;
  }
}

.inline-label {
  width: 180px;
}
</style>
