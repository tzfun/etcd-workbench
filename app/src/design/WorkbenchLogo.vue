<script setup lang="ts">
import etcd from "~/assets/etcd.png"
import {ref} from "vue";
const etcdLogo = ref(etcd)
defineProps({
  matrix: {
    type: Boolean
  },
  logoSize:{
    type: String,
    default: () => {
      return "100px"
    }
  },
  fontSize: {
    type: String,
    default: ()=> {
      return "50px"
    }
  }
})

</script>

<template>
  <div class="design">
    <div class="container-matrix" align="center" v-if="matrix">
      <img :src="etcdLogo" class="logo" :style="`width:${logoSize};`">
      <div class="words" :style="`font-size:${fontSize};line-height:${logoSize};`">Workbench</div>
    </div>
    <div class="container" v-else>
      <img :src="etcdLogo" class="logo" :style="`width:${logoSize};`">
      <div class="words" :style="`font-size:${fontSize};line-height:${logoSize};`">Workbench</div>
    </div>
  </div>

</template>

<style scoped lang="scss">
.design {
  padding: 0;
  margin: 0;
  overflow: hidden;
  position: relative;
  height: max-content;

  .words {
    color: #7c7d87;
    text-align: center;
    font-weight: 800;
    margin-left: 15px;
  }

  .container {
    display: flex;
    justify-content: center;
  }

  .container-matrix {
    position: relative;

    .words {
      margin-left: 0;
    }
  }
}
</style>
