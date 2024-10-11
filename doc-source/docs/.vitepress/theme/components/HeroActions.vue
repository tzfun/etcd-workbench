<script setup>
import {computed} from 'vue'
import {useData} from 'vitepress'
import Windows from './icon/Windows.vue'
import Apple from './icon/Apple.vue'
import {homepage, version, description as webVersion} from '../../../../package.json'
import Java from "./icon/Java.vue";
import Docker from "./icon/Docker.vue";

const {lang, isDark} = useData()

const downloadAppText = computed(() => {
  switch (lang.value) {
    case 'zh':
      return '下载App'
    default:
      return 'Download App'
  }
})

const downloadWebText = computed(() => {
  switch (lang.value) {
    case 'zh':
      return 'Web服务'
    default:
      return 'Web Server'
  }
})

const downloadAppMenu = computed(() => {
  switch (lang.value) {
    case 'zh':
      let gitee = homepage.replace("github", "gitee")
      return [
        {
          text: 'Windows x64安装包 (.exe)',
          link: `${gitee}/releases/download/App-${version}/etcd-workbench-${version}-windows-x86_64.exe`,
          icon: Windows,
        },
        {
          text: 'macOS Apple芯片 (.dmg)',
          link: `${gitee}/releases/download/App-${version}/etcd-workbench-${version}-macos-aarch64.dmg`,
          icon: Apple,
        },
        {
          text: 'macOS Intel芯片 (.dmg)',
          link: `${gitee}/releases/download/App-${version}/etcd-workbench-${version}-macos-x86_64.dmg`,
          icon: Apple,
        },
      ]
    default:
      return [
        {
          text: 'Windows x64 Installer (.exe)',
          link: `${homepage}/releases/download/App-${version}/etcd-workbench-${version}-windows-x86_64.exe`,
          icon: Windows,
        },
        {
          text: 'macOS Apple Silicon (.dmg)',
          link: `${homepage}/releases/download/App-${version}/etcd-workbench-${version}-macos-aarch64.dmg`,
          icon: Apple,
        },
        {
          text: 'macOS Intel (.dmg)',
          link: `${homepage}/releases/download/App-${version}/etcd-workbench-${version}-macos-x86_64.dmg`,
          icon: Apple,
        }
      ]
  }
})

const downloadWebMenu = computed(() => {
  switch (lang.value) {
    case 'zh':
      let gitee = homepage.replace("github", "gitee")
      return [
        {
          text: '下载执行包 (.jar)',
          link: `${gitee}/releases/download/Web-${webVersion}/etcd-workbench-${webVersion}.jar`,
          icon: Java,
        },
        {
          text: 'Docker镜像',
          link: `https://hub.docker.com/r/tzfun/etcd-workbench`,
          icon: Docker,
        },
      ]
    default:
      return [
        {
          text: 'Download Binary Package (.jar)',
          link: `${homepage}/releases/download/Web-${webVersion}/etcd-workbench-${webVersion}.jar`,
          icon: Java,
        },
        {
          text: 'Docker Image',
          link: `https://hub.docker.com/r/tzfun/etcd-workbench`,
          icon: Docker,
        },
      ]
  }
})
</script>

<template>
  <div class="actions">
    <div class="action">
      <a class="action-button brand dropdown-button">{{ downloadAppText }}</a>
      <ul class="dropdown-menu">
        <li v-for="(m, i) in downloadAppMenu" :key="i" style="font-size: 14px">
          <component :is="m.icon"/>
          <a :href="m.link"
             target="_blank">{{ m.text }}</a>
        </li>
      </ul>
    </div>
    <div class="action">
      <a class="action-button alt"
         :href="homepage"
         rel="noreferrer"
         target="_blank">
        {{ downloadWebText }}
      </a>
      <ul class="dropdown-menu dropdown-menu-alt">
        <li v-for="(m, i) in downloadWebMenu" :key="i" style="font-size: 14px">
          <component :is="m.icon" :dark="isDark"/>
          <a :href="m.link"
             target="_blank">{{ m.text }}</a>
        </li>
      </ul>
    </div>
  </div>
</template>

<style scoped>
.actions {
  display: flex;
  flex-wrap: wrap;
  margin: -6px;
  padding-top: 24px;
  justify-content: center;
}

.action {
  flex-shrink: 0;
  padding: 6px;
}

@media (min-width: 640px) {
  .actions {
    padding-top: 32px;
  }
}

@media (min-width: 960px) {
  .actions {
    justify-content: flex-start;
  }
}

.action-button.brand {
  border-color: var(--vp-button-brand-border);
  color: var(--vp-button-brand-text);
  background-color: var(--vp-button-brand-bg);
}

.action-button.alt {
  border-color: var(--vp-button-alt-border);
  color: var(--vp-button-alt-text);
  background-color: var(--vp-button-alt-bg);
}

.action-button {
  border-radius: 20px;
  padding: 0 20px;
  line-height: 38px;
  font-size: 14px;
  display: inline-block;
  border: 1px solid transparent;
  text-align: center;
  font-weight: 600;
  white-space: nowrap;
  transition: color 0.25s, border-color 0.25s, background-color 0.25s;
  cursor: pointer;
}

.action:hover .dropdown-menu {
  display: block;
}

.dropdown-menu {
  position: absolute;
  list-style-type: none;
  margin: 5px 0 0;
  padding: 5px;
  border-radius: 10px;
  border: 1px solid var(--vp-button-brand-border);
  background-color: var(--vp-button-brand-bg);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  z-index: 1;
  display: none;
}

.dropdown-menu-alt {
  background-color: var(--vp-button-alt-bg);
}

.dropdown-menu li {
  padding: 8px 8px;
  font-size: 14px;
  cursor: pointer;
  display: flex;
  flex-direction: row;
  gap: 5px;
  align-items: center;
  color: var(--vp-button-brand-text);
}

.dropdown-menu-alt li {
  color: var(--vp-button-alt-text);
  font-weight: 500;
}

.dropdown-menu li:hover {
  background-color: #f0f0f066;
  border-radius: 10px;
}

.dropdown-menu-alt li:hover {
  background-color: rgba(128, 125, 125, 0.4);
  border-radius: 10px;
}
</style>