<script setup lang="ts">

import {computed, onMounted, PropType, reactive, ref} from "vue";
import {ErrorPayload, SessionData} from "~/common/transport/connection.ts";
import {_compact, _defragment, _getCluster, _handleError, _maintenanceCreateSnapshotTask, _metrics} from "~/common/services.ts";
import {Alarm, Cluster} from "~/common/transport/maintenance.ts";
import {_byteTextFormat, _isEmpty} from "~/common/utils.ts";
import {_alertError, _confirmSystem, _emitLocal, _tipSuccess, _tipWarn, EventName} from "~/common/events.ts";
import {save} from "@tauri-apps/api/dialog";
import {_getDownloadPath} from "~/common/windows.ts";

const props = defineProps({
  session: {
    type: Object as PropType<SessionData>,
    required: true
  }
})
const cluster = ref<Cluster>()
const INFO_COL = {
  xxl: 3,
  xl: 4,
  lg: 4,
  md: 6,
  sm: 6,
  xs: 12
}
const MEMBER_COL = {
  xxl: 3,
  xl: 4,
  lg: 4,
  md: 6,
  sm: 6,
  xs: 12
}

const loadingStore = reactive({
  loadCluster: false,
  defragment: false,
  compact: false,
  snapshot: false,
  metrics: false,
})

const compactDialog = reactive({
  show: false,
  revision: '',
  physical: false,
})

const metricsDialog = reactive({
  data: <Array<string[]>>[],
  keyword: '',
  show: false
})

const computedMetrics = computed(() => {
  const keyword = metricsDialog.keyword
  if (_isEmpty(keyword)) {
    return metricsDialog.data
  }

  const lowerKeyword = keyword.toLocaleLowerCase()

  // 辅助函数：转义正则特殊字符
  function escapeRegExp(string:string) {
    return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  }

  const pattern = new RegExp(`(${escapeRegExp(keyword)})`, 'gi');

  return metricsDialog.data
    .filter(kv => kv[0].toLowerCase().includes(lowerKeyword))
    .map(kv => [kv[0].replace(pattern, `<span class='search-mark'>$1</span>`), kv[1]])
})

onMounted(() => {
  loadCluster()
})

const loadCluster = () => {
  loadingStore.loadCluster = true
  _getCluster(props.session?.id).then(c => {
    cluster.value = c
  }).catch(e => {
    _handleError({
      e,
      session: props.session
    })
  }).finally(() => {
    loadingStore.loadCluster = false
  })
}

const defragment = () => {
  _confirmSystem('Confirm to perform defragmentation?').then(() => {
    loadingStore.defragment = true
    _defragment(props.session?.id).then(() => {
      _tipSuccess("Succeeded!")
    }).catch((e: string | ErrorPayload) => {
      _handleError({
        e,
        session: props.session
      })
    }).finally(() => {
      loadingStore.defragment = false
    })
  }).catch(() => {
  })
}

const showCompactDialog = () => {
  compactDialog.revision = ''
  compactDialog.physical = false
  compactDialog.show = true
}

const compact = () => {
  if (_isEmpty(compactDialog.revision)) {
    _tipWarn("Need a valid revision")
    return
  }

  const revision = parseInt(compactDialog.revision)
  _confirmSystem('Confirm compaction operation?').then(() => {
    loadingStore.compact = true
    _compact(props.session?.id, revision, compactDialog.physical).then(() => {
      _tipSuccess("Succeeded!")
      compactDialog.show = false
    }).catch((e: string | ErrorPayload) => {
      _handleError({
        e,
        session: props.session
      })
    }).finally(() => {
      loadingStore.compact = false
    })
  }).catch(() => {
  })
}

const snapshot = () => {
  _confirmSystem('Confirm snapshot creation? Download duration varies by data size.').then(async () => {
    let downloadPath = await _getDownloadPath()
    save({
      defaultPath: downloadPath
    }).then(filepath => {
      if (filepath) {
        loadingStore.snapshot = true
        _maintenanceCreateSnapshotTask(props.session?.id, filepath).then(info => {
          _emitLocal(EventName.SNAPSHOT_CREATE, info)
        }).catch(e => {
          _handleError({
            e,
            session: props.session
          })
        }).finally(() => {
          loadingStore.snapshot = false
        })
      }
    }).catch(e => {
      _alertError(e)
    })
  }).catch(() => {
  })
}

const showMetricsDialog = () => {
  if (metricsDialog.data.length == 0) {
    loadMetrics().then(() => {
      metricsDialog.show = true 
    }).catch(() => {})
  } else {
    metricsDialog.show = true
  }
}

const loadMetrics = ():Promise<Array<string[]>> => {
  return new Promise<Array<string[]>>((resolve, reject) => {
    loadingStore.metrics = true
    _metrics(props.session?.id).then(data => {
      metricsDialog.data = data
      resolve(data)
    }).catch(e => {
      reject()
      _handleError({
        e,
        session: props.session
      })
    }).finally(() => {
      loadingStore.metrics = false
    })
  })
}
</script>

<template>
  <div class="fill-height pa-5 overflow-y-auto">
    <div>
      <v-btn 
            variant="tonal"
            size="small"
            icon="mdi-refresh"
            @click="loadCluster"
            :loading="loadingStore.loadCluster"
            title="Refresh"
      ></v-btn>
      <v-btn class="text-none ml-2"
             prepend-icon="mdi-database-search"
             @click="showMetricsDialog"
             color="success"
             text="Metrics"
             title="Query metrics data from etcd server."
             :loading="loadingStore.metrics"
      ></v-btn>
      <v-btn class="text-none ml-2"
             prepend-icon="mdi-database-cog"
             @click="showCompactDialog"
             color="red-accent-3"
             text="Compact"
             title="Compacts the event history in the etcd key-value store."
             :loading="loadingStore.compact"
      ></v-btn>
      <v-btn class="text-none ml-2"
             prepend-icon="mdi-database-sync"
             @click="defragment"
             color="yellow"
             text="Defragment"
             title="Defragment backend database to recover storage space."
             :loading="loadingStore.defragment"
      ></v-btn>
      <v-btn class="text-none ml-2"
             prepend-icon="mdi-cloud-arrow-down"
             @click="snapshot"
             color="brown-darken-1"
             text="Snapshot"
             title="Save snapshot from etcd server to local file"
             :loading="loadingStore.snapshot"
      ></v-btn>
    </div>

    <div v-if="cluster" class="d-block">
      <div>
        <v-card class="mx-auto mt-5 mb-5" border flat>
          <v-list-item class="user-select-none">
            <template v-slot:prepend>
              <v-avatar color="surface-light" size="32">🎯</v-avatar>
            </template>

            <template v-slot:title> Cluster Information</template>
          </v-list-item>
          <v-divider></v-divider>
          <v-card-text class=" pa-6">
            <v-row>
              <v-col :xxl="INFO_COL.xxl" :xl="INFO_COL.xl" :lg="INFO_COL.lg" :md="INFO_COL.md" :sm="INFO_COL.sm"
                     :xs="INFO_COL.xs" class="d-flex info-item">
                <div class="info-label text-medium-emphasis">Cluster ID</div>
                <div class="info-value text-high-emphasis">{{ cluster.id }}</div>
              </v-col>
              <v-col :xxl="INFO_COL.xxl" :xl="INFO_COL.xl" :lg="INFO_COL.lg" :md="INFO_COL.md" :sm="INFO_COL.sm"
                     :xs="INFO_COL.xs" class="d-flex info-item">
                <div class="info-label text-medium-emphasis">Member ID</div>
                <div class="info-value text-high-emphasis">{{ cluster.memberId }}</div>
              </v-col>
              <v-col :xxl="INFO_COL.xxl" :xl="INFO_COL.xl" :lg="INFO_COL.lg" :md="INFO_COL.md" :sm="INFO_COL.sm"
                     :xs="INFO_COL.xs" class="d-flex info-item">
                <div class="info-label text-medium-emphasis">Revision</div>
                <div class="info-value text-high-emphasis">{{ cluster.revision }}</div>
              </v-col>
            </v-row>

            <v-divider class="mt-5 mb-5"></v-divider>

            <v-row>
              <v-col :xxl="INFO_COL.xxl" :xl="INFO_COL.xl" :lg="INFO_COL.lg" :md="INFO_COL.md" :sm="INFO_COL.sm"
                     :xs="INFO_COL.xs" class="d-flex info-item">
                <div class="info-label text-medium-emphasis">Etcd Version</div>
                <div class="info-value text-high-emphasis">{{ cluster.status.version }}</div>
              </v-col>
              <v-col :xxl="INFO_COL.xxl" :xl="INFO_COL.xl" :lg="INFO_COL.lg" :md="INFO_COL.md" :sm="INFO_COL.sm"
                     :xs="INFO_COL.xs" class="d-flex info-item">
                <div class="info-label text-medium-emphasis">Leader</div>
                <div class="info-value text-high-emphasis">{{ cluster.status.leader }}</div>
              </v-col>
              <v-col :xxl="INFO_COL.xxl" :xl="INFO_COL.xl" :lg="INFO_COL.lg" :md="INFO_COL.md" :sm="INFO_COL.sm"
                     :xs="INFO_COL.xs" class="d-flex info-item">
                <div class="info-label text-medium-emphasis">DB Size Allocated</div>
                <div class="info-value text-high-emphasis">{{ _byteTextFormat(cluster.status.dbSizeAllocated) }}</div>
              </v-col>
              <v-col :xxl="INFO_COL.xxl" :xl="INFO_COL.xl" :lg="INFO_COL.lg" :md="INFO_COL.md" :sm="INFO_COL.sm"
                     :xs="INFO_COL.xs" class="d-flex info-item">
                <div class="info-label text-medium-emphasis">DB Size Used</div>
                <div class="info-value text-high-emphasis">{{ _byteTextFormat(cluster.status.dbSizeUsed) }}</div>
              </v-col>
            </v-row>

            <v-divider class="mt-5 mb-5"></v-divider>

            <v-row>
              <v-col :xxl="INFO_COL.xxl" :xl="INFO_COL.xl" :lg="INFO_COL.lg" :md="INFO_COL.md" :sm="INFO_COL.sm"
                     :xs="INFO_COL.xs" class="d-flex info-item">
                <div class="info-label text-medium-emphasis">Raft Index</div>
                <div class="info-value text-high-emphasis">{{ cluster.status.raftIndex }}</div>
              </v-col>
              <v-col :xxl="INFO_COL.xxl" :xl="INFO_COL.xl" :lg="INFO_COL.lg" :md="INFO_COL.md" :sm="INFO_COL.sm"
                     :xs="INFO_COL.xs" class="d-flex info-item">
                <div class="info-label text-medium-emphasis">Raft Applied Index</div>
                <div class="info-value text-high-emphasis">{{ cluster.status.raftAppliedIndex }}</div>
              </v-col>
              <v-col :xxl="INFO_COL.xxl" :xl="INFO_COL.xl" :lg="INFO_COL.lg" :md="INFO_COL.md" :sm="INFO_COL.sm"
                     :xs="INFO_COL.xs" class="d-flex info-item">
                <div class="info-label text-medium-emphasis">Raft Term</div>
                <div class="info-value text-high-emphasis">{{ cluster.status.raftTerm }}</div>
              </v-col>
            </v-row>

            <v-expansion-panels variant="accordion" class="mt-5" v-if="cluster.status.errors.length > 0">
              <v-expansion-panel>
                <template v-slot:title>
                  <v-icon color="red" class="mr-2">mdi-alert-circle-outline</v-icon>
                  Errors
                </template>
                <template v-slot:text>
                  <v-list>
                    <v-list-item v-for="(err, idx) in cluster.status.errors" :key="idx">
                      {{ err }}
                    </v-list-item>
                  </v-list>
                </template>
              </v-expansion-panel>
            </v-expansion-panels>
          </v-card-text>
        </v-card>
      </div>
      <div>
        <v-row>
          <v-col :xxl="MEMBER_COL.xxl"
                 :xl="MEMBER_COL.xl"
                 :lg="MEMBER_COL.lg"
                 :md="MEMBER_COL.md"
                 :sm="MEMBER_COL.sm"
                 :xs="MEMBER_COL.xs"
                 v-for="(member,idx) in cluster.members"
                 :key="idx"
          >
            <v-card :title="member.name"
                    class="member-item"
                    border
                    flat
                    hover
            >
              <template v-slot:append>
                <v-tooltip text="Everything is ok!"
                           location="top"
                           v-if="member.alarmType == Alarm.None">
                  <template v-slot:activator="{ props }">
                    <v-icon v-bind="props"
                            color="green-lighten-1">mdi-heart
                    </v-icon>
                  </template>
                </v-tooltip>
                <v-tooltip text="Alarm: space quota is exhausted!"
                           location="top"
                           v-else-if="member.alarmType == Alarm.Nospace">
                  <template v-slot:activator="{ props }">
                    <v-icon v-bind="props"
                            color="red-accent-3">mdi-database-alert
                    </v-icon>
                  </template>
                </v-tooltip>
                <v-tooltip text="Alarm: kv store corruption detected!"
                           location="top"
                           v-else-if="member.alarmType == Alarm.Corrupt">
                  <template v-slot:activator="{ props }">
                    <v-icon v-bind="props"
                            color="red-accent-2">mdi-file-document-alert
                    </v-icon>
                  </template>
                </v-tooltip>
              </template>

              <v-divider></v-divider>
              <v-card-text>
                <div class="member-icon text-center">
                  <svg t="1698415293514"
                       viewBox="0 0 1024 1024"
                       version="1.1"
                       xmlns="http://www.w3.org/2000/svg"
                       p-id="1652"
                       width="100"
                       height="100">
                    <path
                        d="M1024 210.823529C1024 73.908706 760.169412 0 512 0S0 73.908706 0 210.823529c0 8.975059 1.445647 19.154824 4.818824 30.117647H0v572.235295C0 950.091294 263.830588 1024 512 1024s512-73.908706 512-210.823529V240.941176h-4.818824a103.002353 103.002353 0 0 0 4.818824-30.117647zM512 60.235294C770.590118 60.235294 963.764706 139.745882 963.764706 210.823529c0 23.973647-23.371294 50.296471-65.837177 74.029177C812.453647 332.8 668.190118 361.411765 512 361.411765s-300.453647-28.611765-385.927529-76.559059C83.606588 261.12 60.235294 234.797176 60.235294 210.823529 60.235294 139.745882 253.409882 60.235294 512 60.235294z m0 903.529412C253.409882 963.764706 60.235294 884.254118 60.235294 813.176471v-98.364236c10.541176 8.131765 22.106353 16.323765 36.382118 24.274824 94.087529 52.645647 249.374118 84.088471 415.382588 84.08847 20.781176 0 41.381647-0.481882 61.620706-1.445647 142.215529-6.686118 271.36-36.562824 353.701647-82.642823l0.060235-0.060235c14.215529-8.011294 25.901176-16.143059 36.382118-24.274824V813.176471c0 71.077647-193.174588 150.588235-451.764706 150.588235z m385.927529-277.263059c-85.534118 47.887059-229.737412 76.498824-385.927529 76.498824-19.516235 0-38.912-0.481882-57.946353-1.325177-133.360941-6.144-253.168941-33.249882-327.981176-75.113412C83.666824 662.708706 60.235294 636.385882 60.235294 612.412235V513.987765c10.541176 8.131765 22.166588 16.323765 36.442353 24.274823 94.027294 52.645647 249.313882 84.088471 415.322353 84.088471s321.295059-31.442824 415.322353-84.088471c14.275765-7.951059 25.901176-16.143059 36.442353-24.274823v98.42447c0 23.973647-23.431529 50.296471-65.837177 74.089412z m-0.060235-200.824471c-85.473882 47.887059-229.677176 76.498824-385.867294 76.498824s-300.393412-28.611765-385.867294-76.498824C83.666824 461.944471 60.235294 435.561412 60.235294 411.587765V313.163294c10.541176 8.192 22.106353 16.323765 36.382118 24.335059C190.704941 390.204235 345.931294 421.647059 512 421.647059s321.295059-31.442824 415.382588-84.148706c14.275765-8.011294 25.840941-16.143059 36.382118-24.335059v98.424471c0 23.973647-23.431529 50.356706-65.897412 74.089411z"
                        p-id="1653"
                        fill="#1296db"/>
                  </svg>
                </div>
                <p class="text-center ma-5 text-high-emphasis font-weight-bold">
                  {{ member.id }}
                  <v-chip v-if="member.id == cluster.status.leader"
                          color="red"
                          variant="elevated"
                          size="small"
                          density="comfortable"
                          class="ml-2"
                  >leader
                  </v-chip>
                  <v-chip v-if="member.id == cluster.memberId"
                          color="primary"
                          variant="elevated"
                          size="small"
                          density="comfortable"
                          class="ml-2"
                  >current
                  </v-chip>
                </p>

                <v-table class="text-caption" density="compact">
                  <tbody>
                  <tr align="right">
                    <th>
                      <v-icon class="mr-2">mdi-link</v-icon>
                      <span>Peer Uri:</span>
                    </th>

                    <td class="text-high-emphasis">
                      <div v-for="uri in member.peerUri" :key="uri">
                        {{ uri }}
                      </div>
                    </td>
                  </tr>

                  <tr align="right">
                    <th>
                      <v-icon class="mr-2">mdi-link-variant</v-icon>
                      <span>Client Uri:</span>
                    </th>

                    <td class="text-high-emphasis">
                      <div v-for="uri in member.clientUri" :key="uri">
                        {{ uri }}
                      </div>
                    </td>
                  </tr>
                  </tbody>
                </v-table>
              </v-card-text>
            </v-card>
          </v-col>
        </v-row>

      </div>
    </div>

    <!--  Compact弹窗-->
    <v-dialog
        v-model="compactDialog.show"
        persistent
        width="600px"
        scrollable
    >
      <v-card title="Compact">
        <v-card-text>
          Compacts the event history in the etcd key-value store. The key-value store should be periodically compacted 
          or the event history will continue to grow indefinitely.
          <v-layout class="mt-5">
            <v-text-field v-model="compactDialog.revision"
                          label="Revision"
                          type="number" 
                          density="comfortable"
            ></v-text-field>

            <v-checkbox label="Physical" v-model="compactDialog.physical"></v-checkbox>
          </v-layout>
        </v-card-text>
        <v-card-actions>
          <v-btn text="Cancel"
                 variant="text"
                 class="text-none"
                 @click="compactDialog.show = false"
          ></v-btn>

          <v-btn text="Confirm"
                 variant="flat"
                 class="text-none"
                 color="primary"
                 @click="compact"
                 :loading="loadingStore.compact"
          ></v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <v-dialog
        v-model="metricsDialog.show"
        transition="dialog-bottom-transition"
        persistent
        fullscreen
        scrollable
    >
      <v-card class="pb-8">
        <v-card-title>
          <v-layout>
            Metrics
            <v-btn class="text-none ml-2"
                  prepend-icon="mdi-refresh"
                  size="small"
                  @click="loadMetrics"
                  color="success"
                  text="Refresh"
                  title="Query metrics data from etcd server."
                  :loading="loadingStore.metrics"
            ></v-btn>
            <v-spacer></v-spacer>
            <div style="width: 500px;" class="mr-2">
              <v-text-field v-model="metricsDialog.keyword" 
                          placeholder="Type to search"
                          density="compact"
                          clearable
                          hide-details
              ></v-text-field>
            </div>
            <v-spacer></v-spacer>
            <v-btn
              icon="mdi-close"
              size="x-small"
              @click="metricsDialog.show = false"
            ></v-btn>
          </v-layout>
        </v-card-title>

        <v-card-text>
          <v-virtual-scroll :items="computedMetrics" item-height="30" height="100%">
            <template v-slot:default="{ item }">
              <v-list-item>
                <v-layout class="metric-line px-2">
                  <div v-html="item[0]"></div>
                  <v-spacer></v-spacer>
                  <div>{{ item[1] }}</div>
                </v-layout>
                <v-divider class="mt-2"></v-divider>
              </v-list-item>
            </template>
          </v-virtual-scroll>
        </v-card-text>
      </v-card>
    </v-dialog>
  </div>
</template>

<style scoped lang="scss">
.info-item {

  $--info-label-width: 130px;

  .info-label {
    width: $--info-label-width;
    user-select: none;
    cursor: default;
  }

  .info-label:after {
    content: ":";
  }

  .info-value {
    text-align: right;
    width: calc(100% - $--info-label-width);
  }
}

.member-item {
  .node-uri-label {
    min-width: 200px;
  }
}

.metric-line {
  $--metric-line-height: 30px;
  height: $--metric-line-height;
  line-height: $--metric-line-height;
}
.metric-line:hover {
  background-color: rgba(109, 107, 107, .3);

}
</style>

<style>
.search-mark {
  display: inline-block;
  color: black;
  background-color: yellow;
}
</style>
