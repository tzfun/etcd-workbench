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
import {useI18n} from "vue-i18n";

const {t} = useI18n()

const permissionSelections = [
  {
    title: t('main.roles.read'),
    value: RolePermType.Read
  },
  {
    title: t('main.roles.write'),
    value: RolePermType.Write
  },
  {
    title: t('main.roles.readAndWrite'),
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

const getRoleInfo = (role: any) => {
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
  _confirmSystem(`${t('main.roles.revokePermissionConfirm')}: ${role}?`).then(() => {
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
  _confirmSystem(`${t('main.roles.deleteRoleConfirm')} <br/> <strong>${role}</strong>`).then(() => {
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
    _tipWarn(t('main.roles.requiredRoleNameTip'))
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
    _tipWarn(t('main.roles.requiredKeyTip'))
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
      <v-btn 
            v-bind="props"
            variant="tonal"
            size="small"
            icon="mdi-refresh"
            @click="loadAllRoles"
            :loading="loadingStore.loadAllRoles"
            :title="t('common.refresh')"
      />
      <v-btn class="text-none ml-2"
             prepend-icon="mdi-lock-plus"
             @click="openNewRoleDialog"
             color="green"
             :text="t('main.roles.addRole')"
      />
    </div>

    <div>
      <v-expansion-panels class="mt-5"
                          v-model="expendPanel"
                          @update:model-value="getRoleInfo"
                          v-if="roles.length > 0"
      >
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
              />
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
                          {{ t('common.key') }}
                        </th>
                        <th class="text-left font-weight-bold">
                          {{ t('common.prefix') }}
                        </th>
                        <th class="text-left font-weight-bold">
                          {{ t('main.roles.permission') }}
                        </th>
                        <th class="text-left font-weight-bold">
                          {{ t('main.roles.operation') }}
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
                          <span v-if="perm.allKeys" ></span>
                          <span v-else-if="perm.prefix" class="text-blue">{{t('common.yes')}}</span>
                          <span v-else class="text-grey">{{t('common.no')}}</span>
                        </td>
                        <td>
                          <span
                              v-if="perm.permType == RolePermType.Read"
                              class="text-blue-grey-lighten-1 font-weight-bold"
                          >{{ t('main.roles.read') }}</span>
                          <span
                              v-else-if="perm.permType == RolePermType.Write"
                              class="text-brown-lighten-1 font-weight-bold"
                          >{{ t('main.roles.write') }}</span>
                          <span
                              v-else-if="perm.permType == RolePermType.ReadAndWrite"
                              class="text-deep-orange-lighten-2 font-weight-bold"
                          >{{ t('main.roles.readAndWrite') }}</span>
                        </td>
                        <td>
                          <v-btn :text="t('main.roles.revoke')"
                                 color="red"
                                 class="text-none"
                                 size="small"
                                 prepend-icon="mdi-account-details-outline"
                                 @click="revokeRolePermission(currentRoleId, perm, i)"
                                 :loading="loadingStore.revokeRolePerm"
                          />
                        </td>
                      </tr>
                      </tbody>
                    </v-table>
                  </v-card-text>
                </v-card>
                <div class="flex-column align-center mt-auto mb-auto" style="min-width: 352px;">
                  <v-btn color="primary"
                         :text="t('main.roles.grantPermission')"
                         class="text-none ml-5"
                         prepend-icon="mdi-lock"
                         @click="openGrantPermDialog(currentRoleId)"
                         :loading="loadingStore.grantPerm"
                  />
                  <v-btn color="red"
                         :text="t('main.roles.deleteRole')"
                         class="text-none ml-2"
                         prepend-icon="mdi-trash-can-outline"
                         @click="deleteRole(currentRoleId, idx)"
                         :loading="loadingStore.deleteRole"
                  />
                </div>
              </v-layout>
            </div>
            <div v-else>
              <v-empty-state icon="mdi-alert-circle-outline"
                             :headline="t('main.roles.errorStateHeadline')"
                             :text="t('main.roles.errorStateText')"
                             class="user-select-none"
              />
            </div>
          </v-expansion-panel-text>
        </v-expansion-panel>
      </v-expansion-panels>
      <v-empty-state v-else
                     icon="mdi-package-variant"
                     :headline="t('main.roles.emptyStateHeadline')"
                     class="user-select-none"
      />
    </div>

    <!--  Add Role弹窗-->
    <v-dialog
        v-model="newRoleDialog.show"
        persistent
        width="400px"
        scrollable
    >
      <v-card :title="t('main.roles.addRole')">
        <v-card-text>
          <v-layout>
            <v-text-field v-model="newRoleDialog.role"
                          :label="t('main.roles.roleName')"
                          density="comfortable"
                          prepend-inner-icon="mdi-account"
            />
          </v-layout>
        </v-card-text>
        <v-card-actions>
          <v-btn :text="t('common.cancel')"
                 variant="text"
                 class="text-none"
                 @click="newRoleDialog.show = false"
          />

          <v-btn :text="t('common.commit')"
                 variant="flat"
                 class="text-none"
                 color="primary"
                 @click="newRole"
                 :loading="loadingStore.newRole"
          />
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
      <v-card :title="t('main.roles.grantPermission')">
        <v-card-text>
          <v-layout class="mt-5">
            <div class="inline-label input-label">{{ t('main.roles.role') }}:</div>
            <v-text-field v-model="grantPermDialog.role"
                          density="comfortable"
                          prepend-inner-icon="mdi-account"
                          readonly
                          hide-details
                          width="400px"
            />
          </v-layout>

          <v-layout class="mt-5">
            <div class="inline-label checkbox-label">{{ t('main.roles.keyType') }}:</div>
            <v-checkbox v-model="grantPermDialog.perm.allKeys"
                        :label="t('main.roles.allKeys')"
                        hide-details
            />
            <v-checkbox v-model="grantPermDialog.perm.prefix"
                        :label="t('common.prefix')"
                        class="ml-5"
                        hide-details
            />
          </v-layout>

          <v-layout class="mt-5" v-if="!grantPermDialog.perm.allKeys">
            <div class="inline-label input-label">{{ t('common.key') }}:</div>
            <v-text-field v-model="grantPermDialog.perm.key"
                          density="comfortable"
                          prepend-inner-icon="mdi-file-document"
                          :placeholder="t('main.roles.keyPlaceholder')"
                          :hint="t('main.roles.keyHint')"
                          persistent-hint
                          width="500px"
                          clearable
            />
          </v-layout>

          <v-layout class="mt-5">
            <div class="inline-label select-label">{{ t('main.roles.permission') }}:</div>
            <v-select :items="permissionSelections"
                      v-model="grantPermDialog.perm.permType"
            />
          </v-layout>
        </v-card-text>
        <v-card-actions>
          <v-btn :text="t('common.cancel')"
                 variant="text"
                 class="text-none"
                 @click="grantPermDialog.show = false"
          />

          <v-btn :text="t('common.commit')"
                 variant="flat"
                 class="text-none"
                 color="primary"
                 @click="grantPerm"
                 :loading="loadingStore.grantPerm"
          />
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>

<style scoped lang="scss">
.inline-label {
  width: 120px;
}
</style>
