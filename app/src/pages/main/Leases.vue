<script setup lang="ts">
import {onMounted, onUnmounted, PropType, reactive, ref} from "vue";
import {_getLease, _grantLease, _handleError, _leases, _revokeLeases} from "~/common/services.ts";
import {SessionData} from "~/common/transport/connection.ts";
import {LeaseInfo} from "~/common/transport/kv.ts";
import CountDownTimer from "~/components/CountDownTimer.vue";
import {_confirmSystem, _tipWarn} from "~/common/localEvents.ts";
import {_isEmpty} from "~/common/utils.ts";

const props = defineProps({
  session: {
    type: Object as PropType<SessionData>,
    required: true
  }
})

const leases = ref<string[]>([])
const expendPanel = ref([])
const currentLeaseId = ref<string>()
const currentLeaseInfo = ref<LeaseInfo>()
const loadingStore = reactive({
  getInfo: false,
  revoke: false,
  grant: false
})
const grantNewDialog = reactive({
  show: false,
  ttl: '',
  lease: ''
})
const leaseListeners = reactive<Set<any>>(new Set())

onMounted(() => {
  loadAllLeases()
})

onUnmounted(() => {
  clearAllLeaseListener()
})

const loadAllLeases = () => {
  expendPanel.value = []
  clearAllLeaseListener()
  _leases(props.session?.id).then(data => {
    leases.value = data
  })
}

const getLeaseInfo = (leaseId: any) => {
  currentLeaseId.value = leaseId
  currentLeaseInfo.value = undefined
  if (leaseId) {
    loadingStore.getInfo = true
    _getLease(props.session?.id, leaseId).then(info => {
      currentLeaseInfo.value = info
      let timer = setTimeout(() => {
        leaseListeners.delete(timer)

        removeLease(info.id)
      }, info.ttl * 1000)
      leaseListeners.add(timer)
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

const removeLease = (lease: string) => {
  if (currentLeaseId.value == lease) {
    expendPanel.value = []
    currentLeaseId.value = undefined
    currentLeaseInfo.value = undefined
  }
  let idx = leases.value.indexOf(lease)
  if (idx >= 0) {
    leases.value.splice(idx, 1)
  }
}

const revokeLease = (lease: string) => {
  _confirmSystem(`The key bound to this lease will also be deleted. Are you sure you want to delete it?<br/><br/><strong>${lease}</strong>`).then(() => {
    loadingStore.revoke = true
    _revokeLeases(props.session?.id, lease).then(() => {
      removeLease(lease)
    }).catch(e => {
      _handleError({
        e,
        session: props.session
      })
    }).finally(() => {
      loadingStore.revoke = false
    })
  }).catch(() => {

  })
}

const clearAllLeaseListener = () => {
  for (let listener of leaseListeners) {
    clearTimeout(listener)
  }

  leaseListeners.clear()
}

const openGrantNewDialog = () => {
  grantNewDialog.ttl = ''
  grantNewDialog.lease = ''
  grantNewDialog.show = true
}

const grantLease = () => {
  if (_isEmpty(grantNewDialog.ttl)) {
    _tipWarn("Please fill in valid `TTL` parameters")
    return
  }
  let ttl = parseInt(grantNewDialog.ttl)
  if(ttl <= 0) {
    _tipWarn("`TTL` cannot be 0 or negative")
    return
  }
  loadingStore.grant = true
  let lease = _isEmpty(grantNewDialog.lease) || grantNewDialog.lease == '0' ? undefined : grantNewDialog.lease
  _grantLease(props.session?.id, ttl, lease).then(id => {
    leases.value.push(id)
    grantNewDialog.show = false
  }).catch(e => {
    _handleError({
      e,
      session: props.session
    })
  }).finally(() => {
    loadingStore.grant = false
  })
}

</script>

<template>
  <div class="fill-height pa-5 overflow-y-auto">
    <div>
      <v-btn class="text-none"
             prepend-icon="mdi-refresh"
             @click="loadAllLeases"
             variant="outlined"
      >Refresh
      </v-btn>
      <v-btn class="text-none ml-2"
             prepend-icon="mdi-invoice-plus-outline"
             @click="openGrantNewDialog"
             color="green"
      >Grant New
      </v-btn>
    </div>

    <div>
      <v-expansion-panels class="mt-5"
                          v-model="expendPanel"
                          @update:model-value="getLeaseInfo"
                          v-if="leases.length > 0"
      >
        <v-expansion-panel v-for="(lease, idx) in leases"
                           :key="idx"
                           :title="lease"
                           :value="lease"
        >
          <v-expansion-panel-text v-if="currentLeaseId == lease">
            <v-divider></v-divider>
            <div v-if="loadingStore.getInfo" class="pa-6 align-center justify-center d-flex">
              <v-progress-circular
                  color="primary"
                  size="32"
                  indeterminate
              ></v-progress-circular>
            </div>
            <v-container v-else-if="currentLeaseInfo">
              <v-layout class="justify-center align-content-center mx-auto my-auto mt-5 mb-5" style="min-width: 50vw;">
                <v-card border
                        flat
                        min-width="400px"
                >
                  <v-card-text>
                    <v-table class="text-caption" density="compact">
                      <tbody>
                      <tr align="right">
                        <th>
                          <v-icon class="mr-2">mdi-calendar-clock</v-icon>
                          <span>Granted TTL:</span>
                        </th>

                        <td class="text-high-emphasis">
                          {{ currentLeaseInfo.grantedTtl }}
                        </td>
                      </tr>

                      <tr align="right">
                        <th>
                          <v-icon class="mr-2">mdi-link</v-icon>
                          <span>TTL:</span>
                        </th>

                        <td class="text-high-emphasis">
                          <CountDownTimer :value="currentLeaseInfo.ttl"></CountDownTimer>
                        </td>
                      </tr>

                      <tr align="right">
                        <th>
                          <v-icon class="mr-2">mdi-key-chain</v-icon>
                          Keys:
                        </th>

                        <td class="text-high-emphasis">
                          <div v-for="k in currentLeaseInfo.keys"
                               :key="k"
                               class="text-primary"
                          >
                            {{ k }}
                          </div>
                        </td>
                      </tr>
                      </tbody>
                    </v-table>
                  </v-card-text>
                </v-card>
                <div class="flex-column align-center mt-auto mb-auto" style="width: 200px;">
                  <v-btn color="red"
                         text="Delete"
                         class="text-none ml-5"
                         prepend-icon="mdi-trash-can-outline"
                         @click="revokeLease(currentLeaseInfo.id)"
                         :loading="loadingStore.revoke"
                  ></v-btn>
                </div>
              </v-layout>
            </v-container>
            <div v-else>
              <v-empty-state icon="mdi-alert-circle-outline"
                             headline="Something error!"
                             text="Failed to read lease information, please try again."
              ></v-empty-state>
            </div>
          </v-expansion-panel-text>
        </v-expansion-panel>
      </v-expansion-panels>
      <v-empty-state v-else
                     icon="mdi-package-variant"
                     headline="No Leases"
      ></v-empty-state>
    </div>

    <!--  Grant弹窗-->
    <v-dialog
        v-model="grantNewDialog.show"
        persistent
        max-width="500px"
        min-width="200px"
        scrollable
    >
      <v-card title="New Lease">
        <v-card-text>
          <v-layout class="mb-5">
            <span class="grant-form-label">TTL(s): </span>
            <v-text-field v-model="grantNewDialog.ttl"
                          type="number"
                          density="comfortable"
                          prepend-inner-icon="mdi-clock-time-eight"
                          hint="The key expiration time in seconds."
                          persistent-hint
                          width="400px"
            ></v-text-field>
          </v-layout>
          <v-layout class="mb-5">
            <span class="grant-form-label">Custom ID: </span>
            <v-text-field v-model="grantNewDialog.lease"
                          type="number"
                          density="comfortable"
                          prepend-inner-icon="mdi-identifier"
                          hint="ID is the requested ID for the lease. If ID is set to 0 or empty, the lessor chooses an ID."
                          placeholder="Optional"
                          persistent-hint
                          width="400px"
            ></v-text-field>
          </v-layout>
        </v-card-text>
        <v-card-actions>
          <v-btn text="Cancel"
                 variant="text"
                 class="text-none"
                 @click="grantNewDialog.show = false"
          ></v-btn>

          <v-btn text="Confirm"
                 variant="flat"
                 class="text-none"
                 color="primary"
                 @click="grantLease"
                 :loading="loadingStore.grant"
          ></v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>

<style scoped lang="scss">
.grant-form-label {
  display: inline-block;
  width: 180px;
  line-height: 48px;
}
</style>