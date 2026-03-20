<template>
  <div class="footer">
    <div class="footer-main">
      <div class="footer-main-logo">Universal<span style="color: #94a3bf">Stamp</span></div>
      <div class="footer-main-menu">
        <a href="#">{{ $t('footer.menu.about_project') }}</a>
        <a href="#">{{ $t('footer.menu.privacy_policy') }}</a>
        <a href="#">{{ $t('footer.menu.tos') }}</a>
        <a href="https://github.com/lightsing/uts" target="_blank">GitHub</a>
      </div>
    </div>
    <div class="footer-copyright">
      <p>© 2026 Universal Timestamp Project. All rights reserved.</p>
      <p style="display: flex; align-items: center; column-gap: 5px">
        Made with <span style="color: #ef4444; font-size: 22px">♥</span> for a verifiable future.
      </p>
    </div>
  </div>
</template>

<style scoped>
.footer {
  border-top: 1px solid #e2e8f0;
  box-sizing: border-box;
  padding: 0 80px;
  background-color: #f8fafc;
}

.footer-main {
  display: flex;
  justify-content: space-between;
  border-bottom: 1px solid #e2e8f0;
  padding: 40px 0;
}

.footer-main-logo {
  font-size: 18px;
  font-weight: bold;
  color: #334155;
}

.footer-main-menu {
  display: flex;
  column-gap: 20px;
}

.footer-main-menu a {
  text-decoration: none;
  color: #64748b;
}

.footer-main-menu a:hover {
  color: var(--primary-color);
}

.footer-copyright {
  padding: 20px 0;
  font-size: 14px;
  color: #94a3b8;
  display: flex;
  justify-content: space-between;
  align-items: center;
}
</style>
