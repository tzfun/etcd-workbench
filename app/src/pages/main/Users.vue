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
import {useI18n} from "vue-i18n";
import {
  DIALOG_BUTTON_DENSITY,
  DIALOG_BUTTON_SIZE,
  PAGE_BUTTON_SIZE,
  PAGE_REFRESH_BUTTON_SIZE
} from "~/common/vuetify.ts";

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
const {t} = useI18n()
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
  _confirmSystem(t('main.users.authEnableConfirm')).then(() => {
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
  _confirmSystem(t('main.users.authDisableConfirm')).then(() => {
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
  _confirmSystem(t('main.users.revokeUserRoleConfirm', {role, user})).then(() => {
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
  editUserDialog.title = t('main.users.changePassword')
  editUserDialog.newUser = false
  editUserDialog.show = true
}

const changePassword = () => {
  if (_isEmpty(editUserDialog.password)) {
    _tipWarn(t('main.users.requiredPasswordTip'))
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
  editUserDialog.title = t('main.users.addUser')
  editUserDialog.newUser = true
  editUserDialog.show = true
}

const newUser = () => {
  if (_isEmpty(editUserDialog.password)) {
    _tipWarn(t('main.users.requiredPasswordTip'))
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
  _confirmSystem(t('main.users.deleteUserConfirm', {user})).then(() => {
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
    _tipWarn(t('main.users.roleHint'))
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
  <div class="fill-height sub-page overflow-y-auto">
    <div>
      <v-btn 
            v-bind="props"
            variant="tonal"
            :size="PAGE_REFRESH_BUTTON_SIZE"
            icon="mdi-refresh"
            @click="loadAllUser"
            :loading="loadingStore.loadAllUser"
            :title="t('common.refresh')"
      />
      <v-btn class="text-none ml-2"
             prepend-icon="mdi-account-plus-outline"
             @click="openNewUserDialog"
             color="green"
             :size="PAGE_BUTTON_SIZE"
             :text="t('main.users.addUser')"
      />
      <v-btn class="text-none ml-2"
             prepend-icon="mdi-lock"
             @click="authEnable"
             color="yellow"
             :size="PAGE_BUTTON_SIZE"
             :text="t('main.users.authEnable')"
             :loading="loadingStore.authEnable"
      />
      <v-btn class="text-none ml-2"
             prepend-icon="mdi-lock-open-variant"
             @click="authDisable"
             color="red"
             :size="PAGE_BUTTON_SIZE"
             :text="t('main.users.authDisable')"
             :loading="loadingStore.authDisable"
      />
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
                {{ t('main.users.user') }}
              </th>
              <th class="text-left role-col font-weight-bold">
                {{ t('main.users.roles') }}
              </th>
              <th class="text-left op-col">
                <v-text-field
                    v-model="search"
                    density="compact"
                    :label="t('main.users.search')"
                    prepend-inner-icon="mdi-magnify"
                    variant="solo-filled"
                    flat
                    hide-details
                    single-line
                />
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
                            icon="mdi-close-circle"
                    />
                  </template>
                </v-chip>
              </td>
              <td class="op-col">
                <v-btn :text="t('main.users.grantRole')"
                       color="green"
                       class="text-none"
                       size="small"
                       prepend-icon="mdi-account-details-outline"
                       @click="openGrantRoleDialog(user.user,idx)"
                />
                <v-btn :text="t('main.users.changePassword')"
                       color="yellow"
                       class="text-none ml-2"
                       size="small"
                       prepend-icon="mdi-lock-reset"
                       @click="openChangePasswordDialog(user.user)"
                />
                <v-btn :text="t('common.delete')"
                       v-if="user.user != session.user"
                       prepend-icon="mdi-file-document-minus-outline"
                       color="red"
                       class="text-none ml-2"
                       size="small"
                       @click="deleteUser(user.user, idx)"
                />
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
            <span class="inline-label input-label">{{ t('main.users.user') }}: </span>
            <v-text-field v-model="editUserDialog.user"
                          density="comfortable"
                          prepend-inner-icon="mdi-account"
                          hide-details
                          :placeholder="t('main.users.userPlaceholder')"
                          :readonly="!editUserDialog.newUser"
                          width="400px"
            />
          </v-layout>
          <v-layout class="mb-5">
            <span class="inline-label input-label">{{t('main.users.newPassword') }}: </span>
            <v-text-field v-model="editUserDialog.password"
                          type="password"
                          density="comfortable"
                          prepend-inner-icon="mdi-lock-reset"
                          :placeholder="t('main.users.newPasswordPlaceholder')"
                          hide-details
                          width="400px"
            />
          </v-layout>
        </v-card-text>
        <v-card-actions>
          <v-btn :text="t('common.cancel')"
                 variant="text"
                 class="text-none"
                 :size="DIALOG_BUTTON_SIZE"
                 :density="DIALOG_BUTTON_DENSITY"
                 @click="editUserDialog.show = false"
          />

          <v-btn :text="t('common.commit')"
                 variant="flat"
                 class="text-none"
                 color="primary"
                 :size="DIALOG_BUTTON_SIZE"
                 :density="DIALOG_BUTTON_DENSITY"
                 @click="confirmEditUser"
                 :loading="loadingStore.editUser"
          />
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
      <v-card :title="t('main.users.grantRole')">
        <v-card-text>
          <v-select :items="roles"
                    :item-props="roleSelectionProps"
                    v-model="grantRoleDialog.role"
                    :placeholder="t('main.users.rolePlaceholder')"
                    :hint="t('main.users.roleHint')"
                    persistent-hint
                    density="comfortable"
          />
        </v-card-text>
        <v-card-actions>
          <v-btn :text="t('common.cancel')"
                 variant="text"
                 class="text-none"
                 :size="DIALOG_BUTTON_SIZE"
                 :density="DIALOG_BUTTON_DENSITY"
                 @click="grantRoleDialog.show = false"
          />

          <v-btn :text="t('common.commit')"
                 variant="flat"
                 class="text-none"
                 color="primary"
                 :size="DIALOG_BUTTON_SIZE"
                 :density="DIALOG_BUTTON_DENSITY"
                 @click="grantRole"
                 :loading="loadingStore.grantRole"
          />
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
