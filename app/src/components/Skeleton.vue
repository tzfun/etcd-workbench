<script setup lang="ts">

defineProps({
  theme: {
    type: String,
    required: true
  },
  height: {
    type: Number,
    default: 100
  },
  width: {
    type: Number,
    default: 160
  }
})
</script>

<template>
  <div :style="`height:${height}px;width:${width}px;`" class="skeleton-container">
    <div v-if="theme != 'auto'"
         :class="`skeleton fill-height skeleton-${theme}`">
      <div class="header"></div>
      <div class="body">
        <div class="nav">
          <div class="nav-item"></div>
          <div class="nav-item"></div>
          <div class="nav-item"></div>
          <div class="nav-item"></div>
        </div>
        <div class="main">
          <div class="box"></div>
        </div>
      </div>
    </div>
    <div v-else class="fill-height skeleton-auto" >
      <div class="skeleton skeleton-light">
        <div class="header"></div>
        <div class="body">
          <div class="nav">
            <div class="nav-item"></div>
            <div class="nav-item"></div>
            <div class="nav-item"></div>
            <div class="nav-item"></div>
          </div>
        </div>
      </div>
      <div class="skeleton skeleton-dark">
        <div class="header"></div>
        <div class="body">
          <div class="nav">
            <div class="nav-item"></div>
            <div class="nav-item"></div>
            <div class="nav-item"></div>
            <div class="nav-item"></div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">

.skeleton-container {
  border-radius: 10px;
}

.skeleton-light {
  $--skeleton-content-light-color: #e0e0e0;
  background-color: #fafafa;
  .header {
    background-color: $--skeleton-content-light-color;
  }
  .body {
    .nav {
      border-right: 1px solid $--skeleton-content-light-color;
      .nav-item {
        background-color: $--skeleton-content-light-color;
      }
    }
    .main {
      .box {
        background-color: $--skeleton-content-light-color;
      }
    }
  }
}

.skeleton-dark {
  $--skeleton-content-dark-color: #3c3c3c;
  background-color: #212121;
  .header {
    background-color: $--skeleton-content-dark-color;
  }

  .body {
    .nav {
      border-right: 1px solid $--skeleton-content-dark-color;
      .nav-item {
        background-color: $--skeleton-content-dark-color;
      }
    }
    .main {
      .box {
        background-color: $--skeleton-content-dark-color;
      }
    }
  }
}

.skeleton-auto {
  display: flex;
  .skeleton {
    width: 50%;
  }
}

.skeleton {
  border: 1px solid #e0e0e0;

  $--skeleton-header-height: 10px;
  $--skeleton-nav-width: 40px;

  .header {
    height: $--skeleton-header-height;
  }
  .body {
    height: calc(100% - $--skeleton-header-height);
    display: flex;

    .nav {
      width: $--skeleton-nav-width;
      .nav-item {
        width: 80%;
        height: 10px;
        margin-top: 20%;
        margin-left: auto;
        margin-right: auto;
      }
    }
    .main {
      width: calc(100% - $--skeleton-nav-width);
      position: relative;

      .box {
        position: absolute;
        width: 60%;
        height: 60%;
        left: 50%;
        top: 50%;
        margin-left: -30%;
        margin-top: -20%;
      }
    }
  }
}
</style>