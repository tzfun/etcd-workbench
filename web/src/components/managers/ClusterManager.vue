<script setup lang="ts">
import {_defragment, _getCluster, _getMemberStatus} from "~/common/Service";
import {Coin, Connection, Link, More, Refresh} from "@element-plus/icons-vue";
import {MemberStatus} from "~/common/Types";
import {_byteFormat, _endLoading, _startLoading} from "../../common/Util";
import {ElMessage} from "element-plus";

const props = defineProps({
  sessionKey: String
})

onMounted(() => {
  loadCluster()
})
const cluster = ref()
const showStatusDialog = ref(false)
const memberStatus = ref<MemberStatus>()

const loadCluster = () => {
  _getCluster(props.sessionKey!).then(data => {
    cluster.value = data
  })
}

const getStatus = (target: string) => {
  _getMemberStatus(props.sessionKey!, target).then((data: MemberStatus) => {
    memberStatus.value = data
    showStatusDialog.value = true
  })
}

const defragment = (target: string) => {
  ElMessageBox.confirm(
      `Are you sure you want to defragment?`,
      'System',
      {
        confirmButtonText: 'OK',
        cancelButtonText: 'Cancel',
        dangerouslyUseHTMLString: true,
        type: 'warning',
      }
  ).then(() => {
    _startLoading()
    _defragment(props.sessionKey!, target).then(()=> {
      ElMessage({
        showClose: true,
        message: "Succeeded",
        type: 'success',
      })
    }).finally(() => {
      _endLoading()
    })
  }).catch(() => {

  })

}

</script>

<template>
  <div class="page">
    <div class="mb-5 button-list">
      <el-button @click="loadCluster" :icon="Refresh">Refresh</el-button>
      <el-button type="warning" :icon="Coin" @click="defragment('')">Defragment</el-button>
    </div>
    <div v-if="cluster">
      <el-descriptions :column="4" border class="cluster-info">
        <el-descriptions-item label="Cluster ID">
          {{ cluster.clusterId }}
        </el-descriptions-item>
        <el-descriptions-item label="Leader ID">
          {{ cluster.leaderId }}
        </el-descriptions-item>
        <el-descriptions-item label="Current ID">
          {{ cluster.memberId }}
        </el-descriptions-item>
        <el-descriptions-item label="Etcd Version">
          {{ cluster.version }}
        </el-descriptions-item>
        <el-descriptions-item label="DB Size">
          {{ _byteFormat(cluster.dbSize) }}
        </el-descriptions-item>
        <el-descriptions-item label="Raft Index">
          {{ cluster.raftIndex }}
        </el-descriptions-item>
        <el-descriptions-item label="Raft Term">
          {{ cluster.raftTerm }}
        </el-descriptions-item>
        <el-descriptions-item label="Revision">
          {{ cluster.revision }}
        </el-descriptions-item>
      </el-descriptions>
      <div class="node-container mt-4 mb-4">

        <el-space wrap size="large">
          <el-card v-for="member in cluster.members"
                   :key="member.id"
                   shadow="hover"
                   class="node" >
            <template #header>
              <div class="card-header items-center">
                <span>{{ member.name }}</span>

                <el-icon style="cursor: pointer;margin-left: auto;"
                         title="Defragment"
                         color="#af551a"
                         @click="defragment(member.clientUri[member.clientUri.length - 1])">
                  <Coin />
                </el-icon>
                <el-icon style="cursor: pointer; margin-left: 5px;"
                         title="Fetch member status"
                         @click="getStatus(member.clientUri[member.clientUri.length - 1])">
                  <More/>
                </el-icon>
              </div>
            </template>
            <div style="text-align: center">
              <svg viewBox="0 0 1024 1024"
                   xmlns="http://www.w3.org/2000/svg"
                   width="100"
                   height="100">
                <path
                    d="M1024 210.823529C1024 73.908706 760.169412 0 512 0S0 73.908706 0 210.823529c0 8.975059 1.445647 19.154824 4.818824 30.117647H0v572.235295C0 950.091294 263.830588 1024 512 1024s512-73.908706 512-210.823529V240.941176h-4.818824a103.002353 103.002353 0 0 0 4.818824-30.117647zM512 60.235294C770.590118 60.235294 963.764706 139.745882 963.764706 210.823529c0 23.973647-23.371294 50.296471-65.837177 74.029177C812.453647 332.8 668.190118 361.411765 512 361.411765s-300.453647-28.611765-385.927529-76.559059C83.606588 261.12 60.235294 234.797176 60.235294 210.823529 60.235294 139.745882 253.409882 60.235294 512 60.235294z m0 903.529412C253.409882 963.764706 60.235294 884.254118 60.235294 813.176471v-98.364236c10.541176 8.131765 22.106353 16.323765 36.382118 24.274824 94.087529 52.645647 249.374118 84.088471 415.382588 84.08847 20.781176 0 41.381647-0.481882 61.620706-1.445647 142.215529-6.686118 271.36-36.562824 353.701647-82.642823l0.060235-0.060235c14.215529-8.011294 25.901176-16.143059 36.382118-24.274824V813.176471c0 71.077647-193.174588 150.588235-451.764706 150.588235z m385.927529-277.263059c-85.534118 47.887059-229.737412 76.498824-385.927529 76.498824-19.516235 0-38.912-0.481882-57.946353-1.325177-133.360941-6.144-253.168941-33.249882-327.981176-75.113412C83.666824 662.708706 60.235294 636.385882 60.235294 612.412235V513.987765c10.541176 8.131765 22.166588 16.323765 36.442353 24.274823 94.027294 52.645647 249.313882 84.088471 415.322353 84.088471s321.295059-31.442824 415.322353-84.088471c14.275765-7.951059 25.901176-16.143059 36.442353-24.274823v98.42447c0 23.973647-23.431529 50.296471-65.837177 74.089412z m-0.060235-200.824471c-85.473882 47.887059-229.677176 76.498824-385.867294 76.498824s-300.393412-28.611765-385.867294-76.498824C83.666824 461.944471 60.235294 435.561412 60.235294 411.587765V313.163294c10.541176 8.192 22.106353 16.323765 36.382118 24.335059C190.704941 390.204235 345.931294 421.647059 512 421.647059s321.295059-31.442824 415.382588-84.148706c14.275765-8.011294 25.840941-16.143059 36.382118-24.335059v98.424471c0 23.973647-23.431529 50.356706-65.897412 74.089411z"
                    fill="#1296db"/>
              </svg>
            </div>

            <div style="text-align: center;" class="m-4">
              <span>{{ member.id }}</span>
              <span v-if="member.id === cluster.leaderId" class="cluster-tag leader-tag">leader</span>
              <span v-if="member.id === cluster.memberId" class="cluster-tag current-tag">current</span>
            </div>
            <div class="mt-4 mb-4">
              <el-descriptions
                  :column="1"
                  border>
                <el-descriptions-item>
                  <template #label>
                    <el-icon>
                      <Connection/>
                    </el-icon>
                    Peer uri
                  </template>
                  <div v-for="uri in member.peerUri" :key="uri">
                    {{ uri }}
                  </div>
                </el-descriptions-item>
                <el-descriptions-item>
                  <template #label>
                    <el-icon>
                      <Link/>
                    </el-icon>
                    Client uri
                  </template>
                  <div v-for="uri in member.clientUri" :key="uri">
                    {{ uri }}
                  </div>
                </el-descriptions-item>
              </el-descriptions>
            </div>
          </el-card>
        </el-space>

      </div>
    </div>

    <el-dialog
        v-model="showStatusDialog"
        title="Cluster Member Status"
        append-to-body
        width="500px">
      <el-descriptions :column="1" border>
        <el-descriptions-item label="DB Size">
          {{ memberStatus?.dbSize }}
        </el-descriptions-item>
        <el-descriptions-item label="Leader">
          {{ memberStatus?.leader }}
        </el-descriptions-item>
        <el-descriptions-item label="Raft Index">
          {{ memberStatus?.raftIndex }}
        </el-descriptions-item>
        <el-descriptions-item label="Raft Term">
          {{ memberStatus?.raftTerm }}
        </el-descriptions-item>
        <el-descriptions-item label="Version">
          {{ memberStatus?.version }}
        </el-descriptions-item>
      </el-descriptions>
    </el-dialog>

  </div>
</template>

<style lang="scss" scoped>
@import '../../styles/index.scss';

.node-container {
  .node {
    padding: 15px;
    justify-content: center;

    .card-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
    }

    .cluster-tag {
      border-radius: 5px;
      margin: 5px;
      padding: 2px 8px;
      font-size: 13px;
    }

    .current-tag {
      background-color: #6868e7;
      color: white;
    }
    .leader-tag {
      background-color: #f14040;
      color: white;
    }

    .is-bordered-label {
      width: 90px;
    }
  }
}

</style>
