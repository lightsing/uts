<template>
  <n-config-provider :theme-overrides="themeOverrides">
    <div class="app">
      <Header />
      <router-view class="main" />
      <Footer />
    </div>
  </n-config-provider>
</template>

<script setup lang="ts">
import { NConfigProvider } from 'naive-ui'
import Header from '@/components/Header.vue'
import Footer from '@/components/Footer.vue'

const themeOverrides = {
  common: {
    primaryColor: 'var(--primary-color)',
    primaryColorHover: 'var(--primary-color-hover)',
    primaryColorPressed: 'var(--primary-color-pressed)',
    primaryColorSuppl: 'var(--primary-color-suppl)',
    borderRadius: '8px',
  },
}
</script>

<style>
.app {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
}

.main {
  flex: 1;
  box-sizing: border-box;
  padding: 0 80px;
}
</style>
