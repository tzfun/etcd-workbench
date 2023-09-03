<script setup lang="ts">
import {Coin, Grid, Lock, User} from "@element-plus/icons-vue";
import ClusterManager from "~/components/managers/ClusterManager.vue";
import KeyManager from "~/components/managers/KeyManager.vue";
import UserManager from "~/components/managers/UserManager.vue";
import RoleManager from "~/components/managers/RoleManager.vue";

defineProps({
  sessionKey: String,
  root: Boolean
})

const visitedPage = ref({
  'cluster': true
})
const activePage = ref('cluster')
const handleSelect = (key: string, keyPath: string[]) => {
  if (!visitedPage.value[key]) {
    visitedPage.value[key] = true
  }
  activePage.value = key
}

</script>

<template>
  <div class="editor-container">
    <div class="aside">
      <el-menu
          :default-active="activePage"
          class="aside-menu"
          @select="handleSelect">
        <el-menu-item index="cluster">
          <el-icon>
            <Grid/>
          </el-icon>
          <span>Cluster</span>
        </el-menu-item>
        <el-menu-item index="keys">
          <el-icon>
            <Coin/>
          </el-icon>
          <span>Keys</span>
        </el-menu-item>
        <el-menu-item index="users" :disabled="!root">
          <el-icon>
            <User/>
          </el-icon>
          <span>Manage Users</span>
        </el-menu-item>
        <el-menu-item index="roles" :disabled="!root">
          <el-icon>
            <Lock/>
          </el-icon>
          <span>Manage Roles</span>
        </el-menu-item>
      </el-menu>
    </div>
    <div class="body">
      <div v-show="activePage === 'cluster'">
        <ClusterManager :session-key="sessionKey"></ClusterManager>
      </div>
      <div v-show="activePage === 'keys'" style="height: 100%">
        <KeyManager :session-key="sessionKey" v-if="visitedPage['keys']"></KeyManager>
      </div>
      <div v-show="activePage === 'users'" >
        <UserManager :session-key="sessionKey" v-if="visitedPage['users']"></UserManager>
      </div>
      <div v-show="activePage === 'roles'">
        <RoleManager :session-key="sessionKey" v-if="visitedPage['roles']"></RoleManager>
      </div>
    </div>
  </div>
</template>

<style lang="scss" scoped>
@import '../styles/index.scss';

.editor-container {
  width: 100%;
  height: 100%;
  display: flex;

  .aside {
    width: $--ep-custom-aside-width;

    .aside-menu {
      width: $--ep-custom-aside-width;
      height: 100%;
      position: fixed;
      left: 0;
    }
  }

  $--body-padding: 15px;

  .body {
    width: calc(100% - $--ep-custom-aside-width - $--body-padding * 2);
    padding: $--body-padding;
    overflow: auto;
  }
}


</style>
