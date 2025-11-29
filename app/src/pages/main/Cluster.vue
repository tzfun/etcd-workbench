<script setup lang="ts">

import {computed, onMounted, PropType, reactive, ref} from "vue";
import {ErrorPayload, SessionData} from "~/common/transport/connection.ts";
import {_compact, _defragment, _getCluster, _handleError, _maintenanceCreateSnapshotTask, _metrics} from "~/common/services.ts";
import {Alarm, Cluster} from "~/common/transport/maintenance.ts";
import {_byteTextFormat, _isEmpty} from "~/common/utils.ts";
import {_alertError, _confirmSystem, _emitLocal, _tipSuccess, _tipWarn, EventName} from "~/common/events.ts";
import {save, SaveDialogOptions} from "@tauri-apps/api/dialog";
import {_getDownloadPath} from "~/common/windows.ts";
import {useI18n} from "vue-i18n";
import {
  DIALOG_BUTTON_DENSITY,
  DIALOG_BUTTON_SIZE,
  PAGE_BUTTON_SIZE,
  PAGE_REFRESH_BUTTON_SIZE
} from "~/common/vuetify.ts";

const {t} = useI18n()
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
  _confirmSystem(t('main.cluster.defragmentConfirmTip')).then(() => {
    loadingStore.defragment = true
    _defragment(props.session?.id).then(() => {
      _tipSuccess(t('common.successTip'))
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
    _tipWarn(t('main.cluster.needRevisionTip'))
    return
  }

  const revision = parseInt(compactDialog.revision)
  _confirmSystem(t('main.cluster.compactConfirmTip')).then(() => {
    loadingStore.compact = true
    _compact(props.session?.id, revision, compactDialog.physical).then(() => {
      _tipSuccess(t('common.successTip'))
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
  _confirmSystem(t('main.cluster.snapshotConfirmTip')).then(async () => {
    let downloadPath = await _getDownloadPath()
    save(<SaveDialogOptions>{
      title: t('main.cluster.snapshot'),
      defaultPath: downloadPath,
      filters: [
        {
          name: 'Etcd Snapshot',
          extensions: ['esnapshot'],
        }
      ]
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
  <div class="fill-height sub-page overflow-y-auto overflow-x-hidden">
    <div>
      <v-btn 
            variant="tonal"
            :size="PAGE_REFRESH_BUTTON_SIZE"
            icon="mdi-refresh"
            @click="loadCluster"
            :loading="loadingStore.loadCluster"
            :title="t('common.refresh')"
      ></v-btn>
      <v-btn class="text-none ml-2"
             prepend-icon="mdi-database-search"
             @click="showMetricsDialog"
             color="success"
             :size="PAGE_BUTTON_SIZE"
             :text="t('main.cluster.metrics')"
             :title="t('main.cluster.metricsBtnTitle')"
             :loading="loadingStore.metrics"
      ></v-btn>
      <v-btn class="text-none ml-2"
             prepend-icon="mdi-database-cog"
             @click="showCompactDialog"
             color="red-accent-3"
             :size="PAGE_BUTTON_SIZE"
             :text="t('main.cluster.compact')"
             :title="t('main.cluster.compactBtnTitle')"
             :loading="loadingStore.compact"
      ></v-btn>
      <v-btn class="text-none ml-2"
             prepend-icon="mdi-database-sync"
             @click="defragment"
             color="yellow"
             :size="PAGE_BUTTON_SIZE"
             :text="t('main.cluster.defragment')"
             :title="t('main.cluster.defragmentBtnTitle')"
             :loading="loadingStore.defragment"
      ></v-btn>
      <v-btn class="text-none ml-2"
             prepend-icon="mdi-cloud-arrow-down"
             @click="snapshot"
             color="brown-darken-1"
             :size="PAGE_BUTTON_SIZE"
             :text="t('main.cluster.snapshot')"
             :title="t('main.cluster.snapshotBtnTitle')"
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

            <template v-slot:title>{{ t('main.cluster.clusterInfoTitle') }}</template>
          </v-list-item>
          <v-divider></v-divider>
          <v-card-text class=" pa-6">
            <v-row>
              <v-col
                  :xxl="INFO_COL.xxl"
                  :xl="INFO_COL.xl"
                  :lg="INFO_COL.lg"
                  :md="INFO_COL.md"
                  :sm="INFO_COL.sm"
                  :xs="INFO_COL.xs"
                  class="d-flex info-item"
              >
                <div class="info-label text-medium-emphasis">{{ t('main.cluster.clusterId') }}</div>
                <div class="info-value text-high-emphasis">{{ cluster.id }}</div>
              </v-col>
              <v-col
                  :xxl="INFO_COL.xxl"
                  :xl="INFO_COL.xl"
                  :lg="INFO_COL.lg"
                  :md="INFO_COL.md"
                  :sm="INFO_COL.sm"
                  :xs="INFO_COL.xs"
                  class="d-flex info-item"
              >
                <div class="info-label text-medium-emphasis">{{ t('main.cluster.memberId') }}</div>
                <div class="info-value text-high-emphasis">{{ cluster.memberId }}</div>
              </v-col>
              <v-col
                  :xxl="INFO_COL.xxl"
                  :xl="INFO_COL.xl"
                  :lg="INFO_COL.lg"
                  :md="INFO_COL.md"
                  :sm="INFO_COL.sm"
                  :xs="INFO_COL.xs"
                  class="d-flex info-item"
              >
                <div class="info-label text-medium-emphasis">{{ t('common.revision') }}</div>
                <div class="info-value text-high-emphasis">{{ cluster.revision }}</div>
              </v-col>
            </v-row>

            <v-divider class="mt-5 mb-5"></v-divider>

            <v-row>
              <v-col
                  :xxl="INFO_COL.xxl"
                  :xl="INFO_COL.xl"
                  :lg="INFO_COL.lg"
                  :md="INFO_COL.md"
                  :sm="INFO_COL.sm"
                  :xs="INFO_COL.xs"
                  class="d-flex info-item"
              >
                <div class="info-label text-medium-emphasis">{{ t('main.cluster.etcdVersion') }}</div>
                <div class="info-value text-high-emphasis">{{ cluster.status.version }}</div>
              </v-col>
              <v-col
                  :xxl="INFO_COL.xxl"
                  :xl="INFO_COL.xl"
                  :lg="INFO_COL.lg"
                  :md="INFO_COL.md"
                  :sm="INFO_COL.sm"
                  :xs="INFO_COL.xs"
                  class="d-flex info-item"
              >
                <div class="info-label text-medium-emphasis">{{ t('main.cluster.leader') }}</div>
                <div class="info-value text-high-emphasis">{{ cluster.status.leader }}</div>
              </v-col>
              <v-col
                  :xxl="INFO_COL.xxl"
                  :xl="INFO_COL.xl"
                  :lg="INFO_COL.lg"
                  :md="INFO_COL.md"
                  :sm="INFO_COL.sm"
                  :xs="INFO_COL.xs"
                  class="d-flex info-item"
              >
                <div class="info-label text-medium-emphasis">{{ t('main.cluster.dbSizeAllocated') }}</div>
                <div class="info-value text-high-emphasis">{{ _byteTextFormat(cluster.status.dbSizeAllocated) }}</div>
              </v-col>
              <v-col
                  :xxl="INFO_COL.xxl"
                  :xl="INFO_COL.xl"
                  :lg="INFO_COL.lg"
                  :md="INFO_COL.md"
                  :sm="INFO_COL.sm"
                  :xs="INFO_COL.xs"
                  class="d-flex info-item"
              >
                <div class="info-label text-medium-emphasis">{{ t('main.cluster.dbSizeUsed') }}</div>
                <div class="info-value text-high-emphasis">{{ _byteTextFormat(cluster.status.dbSizeUsed) }}</div>
              </v-col>
            </v-row>

            <v-divider class="mt-5 mb-5"></v-divider>

            <v-row>
              <v-col
                  :xxl="INFO_COL.xxl"
                  :xl="INFO_COL.xl"
                  :lg="INFO_COL.lg"
                  :md="INFO_COL.md"
                  :sm="INFO_COL.sm"
                  :xs="INFO_COL.xs"
                  class="d-flex info-item"
              >
                <div class="info-label text-medium-emphasis">{{ t('main.cluster.raftIndex') }}</div>
                <div class="info-value text-high-emphasis">{{ cluster.status.raftIndex }}</div>
              </v-col>
              <v-col
                  :xxl="INFO_COL.xxl"
                  :xl="INFO_COL.xl"
                  :lg="INFO_COL.lg"
                  :md="INFO_COL.md"
                  :sm="INFO_COL.sm"
                  :xs="INFO_COL.xs"
                  class="d-flex info-item"
              >
                <div class="info-label text-medium-emphasis">{{ t('main.cluster.raftAppliedIndex') }}</div>
                <div class="info-value text-high-emphasis">{{ cluster.status.raftAppliedIndex }}</div>
              </v-col>
              <v-col
                  :xxl="INFO_COL.xxl"
                  :xl="INFO_COL.xl"
                  :lg="INFO_COL.lg"
                  :md="INFO_COL.md"
                  :sm="INFO_COL.sm"
                  :xs="INFO_COL.xs"
                  class="d-flex info-item"
              >
                <div class="info-label text-medium-emphasis">{{ t('main.cluster.raftTerm') }}</div>
                <div class="info-value text-high-emphasis">{{ cluster.status.raftTerm }}</div>
              </v-col>
            </v-row>

            <v-expansion-panels
                variant="accordion"
                class="mt-5"
                v-if="cluster.status.errors.length > 0"
            >
              <v-expansion-panel>
                <template v-slot:title>
                  <v-icon color="red" class="mr-2">mdi-alert-circle-outline</v-icon>
                  {{ t('main.cluster.errors') }}
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
                <v-tooltip :text="t('main.cluster.everythingOk')"
                           location="top"
                           v-if="member.alarmType == Alarm.None">
                  <template v-slot:activator="{ props }">
                    <v-icon v-bind="props"
                            color="green-lighten-1">mdi-heart
                    </v-icon>
                  </template>
                </v-tooltip>
                <v-tooltip :text="t('main.cluster.alarmNoSpace')"
                           location="top"
                           v-else-if="member.alarmType == Alarm.Nospace">
                  <template v-slot:activator="{ props }">
                    <v-icon v-bind="props"
                            color="red-accent-3">mdi-database-alert
                    </v-icon>
                  </template>
                </v-tooltip>
                <v-tooltip :text="t('main.cluster.alarmCorrupt')"
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
                  <svg
                      viewBox="0 0 1024 1024"
                      xmlns="http://www.w3.org/2000/svg"
                      width="100"
                      height="100">
                    <path
                        d="M1024 210.823529C1024 73.908706 760.169412 0 512 0S0 73.908706 0 210.823529c0 8.975059 1.445647 19.154824 4.818824 30.117647H0v572.235295C0 950.091294 263.830588 1024 512 1024s512-73.908706 512-210.823529V240.941176h-4.818824a103.002353 103.002353 0 0 0 4.818824-30.117647zM512 60.235294C770.590118 60.235294 963.764706 139.745882 963.764706 210.823529c0 23.973647-23.371294 50.296471-65.837177 74.029177C812.453647 332.8 668.190118 361.411765 512 361.411765s-300.453647-28.611765-385.927529-76.559059C83.606588 261.12 60.235294 234.797176 60.235294 210.823529 60.235294 139.745882 253.409882 60.235294 512 60.235294z m0 903.529412C253.409882 963.764706 60.235294 884.254118 60.235294 813.176471v-98.364236c10.541176 8.131765 22.106353 16.323765 36.382118 24.274824 94.087529 52.645647 249.374118 84.088471 415.382588 84.08847 20.781176 0 41.381647-0.481882 61.620706-1.445647 142.215529-6.686118 271.36-36.562824 353.701647-82.642823l0.060235-0.060235c14.215529-8.011294 25.901176-16.143059 36.382118-24.274824V813.176471c0 71.077647-193.174588 150.588235-451.764706 150.588235z m385.927529-277.263059c-85.534118 47.887059-229.737412 76.498824-385.927529 76.498824-19.516235 0-38.912-0.481882-57.946353-1.325177-133.360941-6.144-253.168941-33.249882-327.981176-75.113412C83.666824 662.708706 60.235294 636.385882 60.235294 612.412235V513.987765c10.541176 8.131765 22.166588 16.323765 36.442353 24.274823 94.027294 52.645647 249.313882 84.088471 415.322353 84.088471s321.295059-31.442824 415.322353-84.088471c14.275765-7.951059 25.901176-16.143059 36.442353-24.274823v98.42447c0 23.973647-23.431529 50.296471-65.837177 74.089412z m-0.060235-200.824471c-85.473882 47.887059-229.677176 76.498824-385.867294 76.498824s-300.393412-28.611765-385.867294-76.498824C83.666824 461.944471 60.235294 435.561412 60.235294 411.587765V313.163294c10.541176 8.192 22.106353 16.323765 36.382118 24.335059C190.704941 390.204235 345.931294 421.647059 512 421.647059s321.295059-31.442824 415.382588-84.148706c14.275765-8.011294 25.840941-16.143059 36.382118-24.335059v98.424471c0 23.973647-23.431529 50.356706-65.897412 74.089411z"
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
                  >{{ t('main.cluster.leaderTag') }}
                  </v-chip>
                  <v-chip v-if="member.id == cluster.memberId"
                          color="primary"
                          variant="elevated"
                          size="small"
                          density="comfortable"
                          class="ml-2"
                  >{{ t('main.cluster.currentTag') }}
                  </v-chip>
                </p>

                <v-table class="text-caption" density="compact">
                  <tbody>
                  <tr align="right">
                    <th>
                      <v-icon class="mr-2">mdi-link</v-icon>
                      <span>{{ t('main.cluster.peerUri') }}:</span>
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
                      <span>{{ t('main.cluster.clientUri') }}:</span>
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
      <v-card :title="t('main.cluster.compact')">
        <v-card-text>
          {{ t('main.cluster.compactNotice') }}
          <v-layout class="mt-5">
            <v-text-field v-model="compactDialog.revision"
                          :label="t('common.revision')"
                          type="number" 
                          density="comfortable"
            />

            <v-checkbox
                :label="t('main.cluster.physical')"
                v-model="compactDialog.physical"
                :title="t('main.cluster.physicalTitle')"
            />
          </v-layout>
        </v-card-text>
        <v-card-actions>
          <v-btn :text="t('common.cancel')"
                 variant="text"
                 class="text-none"
                 :size="DIALOG_BUTTON_SIZE"
                 :density="DIALOG_BUTTON_DENSITY"
                 @click="compactDialog.show = false"
          />

          <v-btn :text="t('common.commit')"
                 variant="flat"
                 class="text-none"
                 color="primary"
                 :size="DIALOG_BUTTON_SIZE"
                 :density="DIALOG_BUTTON_DENSITY"
                 @click="compact"
                 :loading="loadingStore.compact"
          />
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
            {{ t('main.cluster.metrics') }}
            <v-btn class="text-none ml-2"
                  prepend-icon="mdi-refresh"
                  size="small"
                  @click="loadMetrics"
                  color="success"
                  :text="t('common.refresh')"
                  :title="t('main.cluster.metricsBtnTitle')"
                  :loading="loadingStore.metrics"
            />
            <v-spacer></v-spacer>
            <div style="width: 500px;" class="mr-2">
              <v-text-field
                  v-model="metricsDialog.keyword"
                  :placeholder="t('common.typeToSearch')"
                  density="compact"
                  clearable
                  hide-details
              />
            </div>
            <v-spacer/>
            <v-btn
              icon="mdi-close"
              size="x-small"
              @click="metricsDialog.show = false"
            />
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
                <v-divider class="mt-2"/>
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

  $--info-label-width: 180px;

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
