<script setup lang="ts">
import {onMounted, PropType, ref} from "vue";
import {_getLease, _handleError, _leases} from "~/common/services.ts";
import {SessionData} from "~/common/transport/connection.ts";
import {LeaseInfo} from "~/common/transport/kv.ts";
import CountDownTimer from "~/components/CountDownTimer.vue";

const props = defineProps({
  session: {
    type: Object as PropType<SessionData>,
    required: true
  }
})

const leases = ref<string[]>([])
const currentLeaseId = ref<string>()
const currentLeaseInfo = ref<LeaseInfo>()
const loading = ref(false)

onMounted(() => {
  loadAllLeases()
})

const loadAllLeases = ()=> {
  _leases(props.session?.id).then(data => {
    leases.value = data
  })
}

const selected = (leaseId) => {
  currentLeaseId.value = leaseId
  if (leaseId) {
    loading.value = true
    _getLease(props.session?.id, leaseId).then(info => {
      currentLeaseInfo.value = info
    }).catch(e => {
      _handleError({
        e,
        session: props.session
      })
    }).finally(() => {
      loading.value = false
    })
  } else {
    currentLeaseInfo.value = undefined
  }
}

</script>

<template>
  <div class="fill-height pa-5 overflow-y-auto">
    <div>
      <v-btn class="text-none"
             prepend-icon="mdi-refresh"
             @click="loadAllLeases"
             color="primary"
      >Refresh
      </v-btn>
    </div>

    <div>
      <v-expansion-panels class="mt-5" @update:model-value="selected">
        <v-expansion-panel v-for="lease in leases"
                           :key="lease"
                           :title="lease"
                           :value="lease"
        >
          <v-expansion-panel-text v-if="currentLeaseId == lease">
            <v-divider></v-divider>
            <div v-if="loading">
              <v-progress-circular
                  color="primary"
                  size="64"
                  indeterminate
              ></v-progress-circular>
            </div>
            <v-container v-else-if="currentLeaseInfo">
              <v-layout class="justify-center align-content-center w-50 mx-auto my-auto">
                <v-card class="mx-auto mt-5 mb-5 w-75" border flat>
                  <v-card-text>

                    <v-table class="text-caption" density="compact">
                      <tbody>
                      <tr align="right">
                        <th>
                          <v-icon class="mr-2">mdi-calendar-clock</v-icon>
                          <span>Granted TTL:</span>
                        </th>

                        <td class="text-high-emphasis">
                          {{currentLeaseInfo.grantedTtl}}
                        </td>
                      </tr>

                      <tr align="right">
                        <th>
                          <v-icon class="mr-2">mdi-link</v-icon>
                          <span>TTL:</span>
                        </th>

                        <td class="text-high-emphasis">
                          <v-icon class="mr-1">mdi-clock-time-four-outline</v-icon>
                          <CountDownTimer :value="currentLeaseInfo.ttl"></CountDownTimer>
                        </td>
                      </tr>

                      <tr align="right">
                        <th><v-icon class="mr-2">mdi-key-chain</v-icon>Keys:</th>

                        <td class="text-high-emphasis">
                          <div v-for="k in currentLeaseInfo.keys" :key="k">
                            {{ k }}
                          </div>
                        </td>
                      </tr>
                      </tbody>
                    </v-table>

                  </v-card-text>
                </v-card>
                <v-layout class="d-flex flex-column align-center mt-auto mb-auto">
                  <v-btn color="red"
                         text="Delete"
                         class="text-none"
                         prepend-icon="mdi-trash-can-outline"
                  ></v-btn>
                </v-layout>
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
    </div>

  </div>
</template>

<style scoped lang="scss">

</style>