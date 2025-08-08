<script setup lang="ts">
import {onMounted, PropType, reactive, ref} from "vue";
import {_getLease, _grantLease, _handleError, _leases, _revokeLeases} from "~/common/services.ts";
import {SessionData} from "~/common/transport/connection.ts";
import {LeaseInfo} from "~/common/transport/kv.ts";
import CountDownTimer from "~/components/CountDownTimer.vue";
import {_confirmSystem, _copyToClipboard, _tipInfo, _tipWarn} from "~/common/events.ts";
import {_isEmpty} from "~/common/utils.ts";
import {useLocale} from "vuetify";

const {t} = useLocale()
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

onMounted(() => {
  loadAllLeases()
})

const loadAllLeases = () => {
  expendPanel.value = []
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
      if (currentLeaseId.value == leaseId) {
        currentLeaseInfo.value = info
      }

      if (!info) {
        removeLease(leaseId)
        _tipInfo(t('main.leases.leaseExpiredTip'))
      }
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
  _confirmSystem(`${t('main.leases.revokeLeaseConfirm')}<br/><br/><strong>${lease}</strong>`).then(() => {
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

const openGrantNewDialog = () => {
  grantNewDialog.ttl = ''
  grantNewDialog.lease = ''
  grantNewDialog.show = true
}

const grantLease = () => {
  if (_isEmpty(grantNewDialog.ttl)) {
    _tipWarn(t('main.leases.requiredTtlTip'))
    return
  }
  let ttl = parseInt(grantNewDialog.ttl)
  if (ttl < 0) {
    _tipWarn(t('main.leases.invalidTtlTip'))
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
      <v-btn
          v-bind="props"
          variant="tonal"
          size="small"
          icon="mdi-refresh"
          @click="loadAllLeases"
          :title="t('common.refresh')"
      />
      <v-btn class="text-none ml-2"
             prepend-icon="mdi-invoice-plus-outline"
             @click="openGrantNewDialog"
             color="green"
             :text="t('main.leases.grantNew')"
      />
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
            <v-divider/>
            <div v-if="loadingStore.getInfo" class="pa-6 align-center justify-center d-flex">
              <v-progress-circular
                  color="primary"
                  size="32"
                  indeterminate
              />
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
                        <th class="table-label">
                          <v-icon class="mr-2" color="teal-darken-1">mdi-lightbulb</v-icon>
                          <span>{{ t('main.leases.leaseId') }}:</span>
                        </th>

                        <td class="text-high-emphasis">
                          <span @click="_copyToClipboard(currentLeaseInfo.id)"
                                class="cursor-pointer text-primary"
                                :title="t('common.copy')"
                          >{{ currentLeaseInfo.id }}</span>
                        </td>
                      </tr>
                      <tr align="right">
                        <th class="table-label">
                          <v-icon class="mr-2" color="teal-darken-1">mdi-calendar-clock</v-icon>
                          <span>{{ t('main.leases.grantedTtl') }}:</span>
                        </th>

                        <td class="text-high-emphasis">
                          {{ currentLeaseInfo.grantedTtl }}
                        </td>
                      </tr>

                      <tr align="right">
                        <th class="table-label">
                          <v-icon class="mr-2" color="teal-darken-1">mdi-link</v-icon>
                          <span>{{ t('common.ttl') }}:</span>
                        </th>

                        <td class="text-high-emphasis">
                          <CountDownTimer
                              :value="currentLeaseInfo.ttl"
                              :key="currentLeaseInfo.ttl"
                          />
                        </td>
                      </tr>

                      <tr align="right">
                        <th class="table-label">
                          <v-icon class="mr-2" color="teal-darken-1">mdi-file-document-multiple</v-icon>
                          {{ t('common.keys') }}:
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
                <div class="flex-column align-center mt-auto mb-auto" style="width: 300px;">
                  <v-btn class="text-none ml-5"
                         icon="mdi-refresh"
                         size="small"
                         @click="getLeaseInfo(currentLeaseInfo.id)"
                         :loading="loadingStore.revoke"
                  />
                  <v-btn color="red"
                         :text="t('common.delete')"
                         class="text-none ml-5"
                         prepend-icon="mdi-trash-can-outline"
                         @click="revokeLease(currentLeaseInfo.id)"
                         :loading="loadingStore.revoke"
                  />
                </div>
              </v-layout>
            </v-container>
            <div v-else>
              <v-empty-state icon="mdi-alert-circle-outline"
                             :headline="t('main.leases.errorStateHeadline')"
                             :text="t('main.leases.errorStateText')"
                             class="user-select-none"
              />
            </div>
          </v-expansion-panel-text>
        </v-expansion-panel>
      </v-expansion-panels>
      <v-empty-state v-else
                     icon="mdi-package-variant"
                     :headline="t('main.leases.emptyStateHeadline')"
                     class="user-select-none"
      />
    </div>

    <!--  Grant弹窗-->
    <v-dialog
        v-model="grantNewDialog.show"
        persistent
        max-width="500px"
        min-width="200px"
        scrollable
    >
      <v-card :title="t('main.leases.grantNew')">
        <v-card-text>
          <v-layout class="mb-5">
            <span class="inline-label input-label">{{ t('common.ttl') }}(s): </span>
            <v-text-field v-model="grantNewDialog.ttl"
                          type="number"
                          density="comfortable"
                          prepend-inner-icon="mdi-clock-time-eight"
                          :hint="t('main.leases.ttlHint')"
                          persistent-hint
                          width="400px"
            />
          </v-layout>
          <v-layout class="mb-5">
            <span class="inline-label input-label">{{ t('main.leases.customId') }}: </span>
            <v-text-field v-model="grantNewDialog.lease"
                          type="number"
                          density="comfortable"
                          prepend-inner-icon="mdi-identifier"
                          :hint="t('main.leases.customIdHint')"
                          :placeholder="t('main.leases.customIdPlaceholder')"
                          persistent-hint
                          width="400px"
            />
          </v-layout>
        </v-card-text>
        <v-card-actions>
          <v-btn :text="t('common.cancel')"
                 variant="text"
                 class="text-none"
                 @click="grantNewDialog.show = false"
          />

          <v-btn :text="t('common.commit')"
                 variant="flat"
                 class="text-none"
                 color="primary"
                 @click="grantLease"
                 :loading="loadingStore.grant"
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
.table-label {
  display: flex;
  align-items: center;
}
</style>