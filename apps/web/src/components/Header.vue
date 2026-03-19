<template>
  <div class="header">
    <div class="header-logo">
      <img src="@/assets/uts-logo.svg" alt="uts-logo" />
      <div class="header-logo-text">
        Universal<span style="color: var(--primary-color)">Stamp</span>
      </div>
    </div>
    <div class="header-menu">
      <a href="#">{{ $t('header.menu.timestamp_verification') }}</a>
      <a href="#">{{ $t('header.menu.upgrade_timestamp') }}</a>
      <a href="#">{{ $t('header.menu.on_chain_proof') }}</a>
      <a href="https://docs.timestamps.now" target="_blank">{{
        $t('header.menu.documentation')
      }}</a>
    </div>
    <div class="header-actions">
      <n-button type="primary">{{ $t('header.actions.connect_wallet') }}</n-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { NButton } from 'naive-ui'
</script>

<style scoped>
.header {
  height: 68px;
  width: 100%;
  display: flex;
  border-bottom: 1px solid #e5e7eb;
  align-items: center;
  box-sizing: border-box;
  padding: 0 80px;
  column-gap: 20px;
}

.header-logo {
  display: flex;
  align-items: center;
  column-gap: 10px;
}

.header-logo img {
  width: 42px;
  height: 42px;
}

.header-logo-text {
  font-size: 20px;
  font-weight: bold;
}

.header-menu {
  flex: 1;
  display: flex;
  column-gap: 40px;
  justify-content: center;
}

.header-menu a {
  text-decoration: none;
  color: #374151;
}

.header-menu a:hover {
  color: var(--primary-color);
}
</style>
