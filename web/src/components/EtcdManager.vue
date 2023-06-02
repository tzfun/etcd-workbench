<script setup lang="ts">
import {ref} from "vue";
import {Coin, FolderChecked, Grid, User, View} from "@element-plus/icons-vue";
import ClusterManager from "~/components/managers/ClusterManager.vue";
import KeyManager from "~/components/managers/KeyManager.vue";
import UserManager from "~/components/managers/UserManager.vue";
import RoleManager from "~/components/managers/RoleManager.vue";

defineProps({
  sessionKey: String
})
const activePage = ref('1')
const handleSelect = (key: string, keyPath: string[]) => {
  activePage.value = key
}

</script>

<template>
  <div class="editor-container">
    <div class="aside">
      <el-menu
          :default-active="activePage"
          class="aside-menu"
          @select="handleSelect"
      >
        <el-menu-item index="1">
          <el-icon>
            <Grid/>
          </el-icon>
          <span>Cluster</span>
        </el-menu-item>
        <el-menu-item index="2">
          <el-icon>
            <Coin/>
          </el-icon>
          <span>Keys</span>
        </el-menu-item>
        <el-menu-item index="3">
          <el-icon>
            <FolderChecked/>
          </el-icon>
          <span>Leases</span>
        </el-menu-item>
        <el-menu-item index="4">
          <el-icon>
            <User/>
          </el-icon>
          <span>Manage Users</span>
        </el-menu-item>
        <el-menu-item index="5" disabled>
          <el-icon>
            <View/>
          </el-icon>
          <span>Manage Roles</span>
        </el-menu-item>
      </el-menu>
    </div>
    <div class="body">
      <div v-show="activePage=== '1'">
        <ClusterManager :session-key="sessionKey"></ClusterManager>
      </div>
      <div v-show="activePage === '2'">
        <KeyManager :session-key="sessionKey"></KeyManager>
      </div>
      <div v-show="activePage === '3'">

      </div>
      <div v-show="activePage === '4'">
        <UserManager :session-key="sessionKey"></UserManager>
      </div>
      <div v-show="activePage === '5'">
        <RoleManager :session-key="sessionKey"></RoleManager>
      </div>
    </div>
  </div>
</template>

<style scoped>
.editor-container {
  width: 100%;
  height: 100%;
  display: flex;
}

.aside, .aside-menu {
  width: var(--ep-custom-aside-width);
  height: 100%;
}

.body {
  width: calc(100% - var(--ep-custom-aside-width));
  height: 100%;
  padding: 15px;
}

.aside-menu {
  position: fixed;
  left: 0;
}
</style>