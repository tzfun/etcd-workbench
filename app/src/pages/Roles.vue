<script setup lang="ts">
import {onMounted, PropType, reactive, ref} from "vue";
import {SessionData} from "~/common/transport/connection.ts";
import {
  _addRole,
  _deleteRole,
  _getAllRoles,
  _getRolePermissions,
  _grantRolePermissions,
  _handleError,
  _revokeRolePermissions
} from "~/common/services.ts";
import {RolePermission, RolePermType} from "~/common/transport/user.ts";
import {_confirmSystem, _tipWarn} from "~/common/events.ts";
import {_isEmpty} from "~/common/utils.ts";

const permissionSelections = [
  {
    title: 'Read',
    value: RolePermType.Read
  },
  {
    title: 'Write',
    value: RolePermType.Write
  },
  {
    title: 'Read And Write',
    value: RolePermType.ReadAndWrite
  }
]

const props = defineProps({
  session: {
    type: Object as PropType<SessionData>,
    required: true
  }
})
const loadingStore = reactive({
  loadAllRoles: false,
  getInfo: false,
  revokeRolePerm: false,
  deleteRole: false,
  newRole: false,
  grantPerm: false
})
const roles = ref<string[]>([])
const expendPanel = ref([])
const currentRoleId = ref<string>()
const currentRoleInfo = ref<RolePermission[]>()

const newRoleDialog = reactive({
  show: false,
  role: ''
})

const grantPermDialog = reactive({
  show: false,
  role: '',
  perm: <RolePermission>{
    key: '',
    permType: RolePermType.Read,
    prefix: false,
    allKeys: false
  }
})

onMounted(() => {
  loadAllRoles()
})

const loadAllRoles = () => {
  expendPanel.value = []
  currentRoleId.value = undefined
  currentRoleInfo.value = undefined
  loadingStore.loadAllRoles = true
  _getAllRoles(props.session?.id).then(data => {
    roles.value = data
  }).catch(e => {
    _handleError({
      e,
      session: props.session
    })
  }).finally(() => {
    loadingStore.loadAllRoles = false
  })
}

const openNewRoleDialog = () => {
  newRoleDialog.role = ''
  newRoleDialog.show = true
}

const getRoleInfo = (role: string) => {
  currentRoleId.value = role
  currentRoleInfo.value = undefined
  if (role) {
    loadingStore.getInfo = true
    _getRolePermissions(props.session?.id, role).then((permissions) => {
      currentRoleInfo.value = permissions
    }).catch(e => {
      _handleError({
        e,
        session: props.session
      })
    }).finally(() => {
      loadingStore.getInfo = false
    })
  }
}

const revokeRolePermission = (role: string, perm: RolePermission, permIdx: number) => {
  _confirmSystem(`Confirm to revoke this permission from role ${role}?`).then(() => {
    loadingStore.revokeRolePerm = true
    _revokeRolePermissions(props.session?.id, role, perm).then(() => {
      if (currentRoleInfo.value) {
        currentRoleInfo.value.splice(permIdx, 1)
      }
    }).catch(e => {
      _handleError({
        e,
        session: props.session
      })
    }).finally(() => {
      loadingStore.revokeRolePerm = false
    })
  }).catch(() => {

  })
}

const deleteRole = (role: string, roleIdx: number) => {
  _confirmSystem(`Confirm to delete this role? <br/> <strong>${role}</strong>`).then(() => {
    loadingStore.deleteRole = true
    _deleteRole(props.session?.id, role).then(() => {
      if (currentRoleId.value == role) {
        expendPanel.value = []
        currentRoleId.value = undefined
        currentRoleInfo.value = undefined
      }
      roles.value.splice(roleIdx, 1)
    }).catch(e => {
      _handleError({
        e,
        session: props.session
      })
    }).finally(() => {
      loadingStore.deleteRole = false
    })
  }).catch(() => {
  })
}

const newRole = () => {
  let role = newRoleDialog.role
  if(_isEmpty(role)) {
    _tipWarn("Role name can not be empty")
    return
  }
  loadingStore.newRole = true
  _addRole(props.session?.id, role).then(() => {
    roles.value.push(role)
    newRoleDialog.show = false
  }).catch(e => {
    _handleError({
      e,
      session: props.session
    })
  }).finally(() => {
    loadingStore.newRole = false
  })
}

const openGrantPermDialog = (role: string) => {
  grantPermDialog.role = role
  grantPermDialog.perm = {
    key: '',
    permType: RolePermType.Read,
    prefix: false,
    allKeys: false
  }
  grantPermDialog.show = true
}

const grantPerm = () => {
  let role = grantPermDialog.role
  let permission = grantPermDialog.perm

  if (!permission.allKeys && _isEmpty(permission.key)) {
    _tipWarn('Key cannot be empty')
    return
  }

  loadingStore.grantPerm = true
  _grantRolePermissions(props.session?.id, role, permission).then(() => {
    if (currentRoleId.value == role) {
      currentRoleInfo.value?.push(permission)
    }
    grantPermDialog.show = false
  }).catch(e => {
    _handleError({
      e,
      session: props.session
    })
  }).finally(() => {
    loadingStore.grantPerm = false
  })
}

</script>

<template>
  <div class="fill-height pa-5 overflow-y-auto">
    <div>
      <v-btn class="text-none"
             prepend-icon="mdi-refresh"
             @click="loadAllRoles"
             variant="outlined"
             text="Refresh"
             :loading="loadingStore.loadAllRoles"
      ></v-btn>
      <v-btn class="text-none ml-2"
             prepend-icon="mdi-lock-plus"
             @click="openNewRoleDialog"
             color="green"
             text="Add Role"
      ></v-btn>
    </div>

    <div>
      <v-expansion-panels class="mt-5"
                          v-model="expendPanel"
                          @update:model-value="getRoleInfo">
        <v-expansion-panel v-for="(role, idx) in roles"
                           :key="idx"
                           :value="role"
        >
          <template #title>
            <v-icon class="mr-2" color="primary">mdi-account</v-icon>
            <span class="font-weight-bold">{{ role }}</span>
          </template>
          <v-expansion-panel-text v-if="currentRoleId == role">
            <v-divider></v-divider>
            <div v-if="loadingStore.getInfo" class="pa-6 align-center justify-center d-flex">
              <v-progress-circular
                  color="primary"
                  size="32"
                  indeterminate
              ></v-progress-circular>
            </div>
            <div v-else-if="currentRoleInfo">
              <v-layout class="justify-center align-content-center mx-auto my-auto mt-5 mb-5" style="min-width: 50vw;">
                <v-card border
                        flat
                        min-width="400px"
                >
                  <v-card-text>
                    <v-table hover>
                      <thead>
                      <tr>
                        <th class="text-left font-weight-bold">
                          Key
                        </th>
                        <th class="text-left font-weight-bold">
                          Prefix
                        </th>
                        <th class="text-left font-weight-bold">
                          Permission
                        </th>
                        <th class="text-left font-weight-bold">
                          Operation
                        </th>
                      </tr>
                      </thead>
                      <tbody>

                      <tr
                          v-for="(perm, i) in currentRoleInfo"
                          :key="i"
                      >
                        <td>
                          <span v-if="perm.allKeys" class="text-red font-weight-bold">ALL KEYS</span>
                          <span v-else>{{ perm.key }}</span>
                        </td>
                        <td>
                          <span v-if="perm.allKeys" >/</span>
                          <span v-else-if="perm.prefix" class="text-blue">Yes</span>
                          <span v-else class="text-grey">No</span>
                        </td>
                        <td>
                          <span v-if="perm.permType == RolePermType.Read" class="text-blue-grey-lighten-1 font-weight-bold">Read</span>
                          <span v-else-if="perm.permType == RolePermType.Write" class="text-brown-lighten-1 font-weight-bold">Write</span>
                          <span v-else-if="perm.permType == RolePermType.ReadAndWrite" class="text-deep-orange-lighten-2 font-weight-bold">Read And Write</span>
                        </td>
                        <td>
                          <v-btn text="Revoke"
                                 color="red"
                                 class="text-none"
                                 size="small"
                                 prepend-icon="mdi-account-details-outline"
                                 @click="revokeRolePermission(currentRoleId, perm, i)"
                                 :loading="loadingStore.revokeRolePerm"
                          ></v-btn>
                        </td>
                      </tr>
                      </tbody>
                    </v-table>
                  </v-card-text>
                </v-card>
                <div class="flex-column align-center mt-auto mb-auto" style="min-width: 352px;">
                  <v-btn color="primary"
                         text="Grant Permission"
                         class="text-none ml-5"
                         prepend-icon="mdi-lock"
                         @click="openGrantPermDialog(currentRoleId)"
                         :loading="loadingStore.grantPerm"
                  ></v-btn>
                  <v-btn color="red"
                         text="Delete Role"
                         class="text-none ml-2"
                         prepend-icon="mdi-trash-can-outline"
                         @click="deleteRole(currentRoleId, idx)"
                         :loading="loadingStore.deleteRole"
                  ></v-btn>
                </div>
              </v-layout>
            </div>
            <div v-else>
              <v-empty-state icon="mdi-alert-circle-outline"
                             headline="Something error!"
                             text="Failed to read lease information, please try again."
              ></v-empty-state>
            </div>
          </v-expansion-panel-text>
        </v-expansion-panel>
      </v-expansion-panels>
    </div>



    <!--  Add Role弹窗-->
    <v-dialog
        v-model="newRoleDialog.show"
        persistent
        width="400px"
        scrollable
    >
      <v-card title="New Role">
        <v-card-text>
          <v-layout>
            <v-text-field v-model="newRoleDialog.role"
                          label="Role Name"
                          density="comfortable"
                          prepend-inner-icon="mdi-account"
            ></v-text-field>
          </v-layout>
        </v-card-text>
        <v-card-actions>
          <v-btn text="Cancel"
                 variant="text"
                 class="text-none"
                 @click="newRoleDialog.show = false"
          ></v-btn>

          <v-btn text="Confirm"
                 variant="flat"
                 class="text-none"
                 color="primary"
                 @click="newRole"
                 :loading="loadingStore.newRole"
          ></v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>



    <!--  Grant Permission弹窗-->
    <v-dialog
        v-model="grantPermDialog.show"
        persistent
        width="700px"
        scrollable
    >
      <v-card title="Grant Permission">
        <v-card-text>
          <v-layout class="mt-5">
            <div class="grant-form-label">Role:</div>
            <v-text-field v-model="grantPermDialog.role"
                          density="comfortable"
                          prepend-inner-icon="mdi-account"
                          readonly
                          hide-details
                          width="400px"
            ></v-text-field>
          </v-layout>

          <v-layout class="mt-5">
            <div class="grant-form-label">Key Type:</div>
            <v-checkbox v-model="grantPermDialog.perm.allKeys"
                        label="All Keys"
                        hide-details
            ></v-checkbox>
            <v-checkbox v-model="grantPermDialog.perm.prefix"
                        label="Prefix"
                        class="ml-5"
                        hide-details
            ></v-checkbox>
          </v-layout>

          <v-layout class="mt-5" v-if="!grantPermDialog.perm.allKeys">
            <div class="grant-form-label">Key:</div>
            <v-text-field v-model="grantPermDialog.perm.key"
                          density="comfortable"
                          prepend-inner-icon="mdi-key"
                          placeholder="Key path"
                          hint="The key here is for the full path and is not related to the namespace of the current connection."
                          persistent-hint
                          width="500px"
                          clearable
            ></v-text-field>
          </v-layout>

          <v-layout class="mt-5">
            <div class="grant-form-label">Permission:</div>
            <v-select :items="permissionSelections"
                      v-model="grantPermDialog.perm.permType"
            >
            </v-select>
          </v-layout>
        </v-card-text>
        <v-card-actions>
          <v-btn text="Cancel"
                 variant="text"
                 class="text-none"
                 @click="grantPermDialog.show = false"
          ></v-btn>

          <v-btn text="Confirm"
                 variant="flat"
                 class="text-none"
                 color="primary"
                 @click="grantPerm"
                 :loading="loadingStore.grantPerm"
          ></v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>

<style scoped lang="scss">
.grant-form-label {
  display: inline-block;
  width: 120px;
  line-height: 48px;
}
</style>
