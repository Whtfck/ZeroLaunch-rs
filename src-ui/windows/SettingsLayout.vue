<template>
  <div class="settings-layout" :class="{ dark: is_dark }">
    <div class="sidebar">
      <div class="header">
        <img
          src="../assets/icon.svg"
          alt="Logo"
          class="logo"
        >
        <span class="title">{{ t('settings.title') }}</span>
      </div>

      <el-scrollbar>
        <el-menu
          :default-active="activeMenu"
          class="settings-menu"
          :collapse="false"
          @select="handleMenuSelect"
        >
          <el-menu-item index="/setting_window/general">
            <el-icon>
              <Setting />
            </el-icon>
            <span>{{ t('settings.menu.general') }}</span>
          </el-menu-item>

          <el-sub-menu index="/setting_window/appearance">
            <template #title>
              <el-icon>
                <Brush />
              </el-icon>
              <span>{{ t('settings.menu.appearance') }}</span>
            </template>
            <el-menu-item index="/setting_window/appearance/search">
              {{
                t('ui_config.search_and_result_settings') }}
            </el-menu-item>
            <el-menu-item index="/setting_window/appearance/background">
              {{
                t('ui_config.background_image_settings')
              }}
            </el-menu-item>
            <el-menu-item index="/setting_window/appearance/window">
              {{ t('ui_config.window_settings')
              }}
            </el-menu-item>
          </el-sub-menu>

          <el-sub-menu index="/setting_window/programs">
            <template #title>
              <el-icon>
                <Search />
              </el-icon>
              <span>{{ t('settings.menu.program_search') }}</span>
            </template>
            <el-menu-item index="/setting_window/programs/paths">
              {{ t('program_index.set_search_path')
              }}
            </el-menu-item>
            <el-menu-item index="/setting_window/programs/blocklist">
              {{ t('program_index.set_blocked_paths')
              }}
            </el-menu-item>
            <el-menu-item index="/setting_window/programs/keywords">
              {{ t('program_index.set_fixed_offset')
              }}
            </el-menu-item>
            <el-menu-item index="/setting_window/programs/aliases">
              {{ t('program_index.setting_alias')
              }}
            </el-menu-item>
            <el-menu-item index="/setting_window/programs/advanced">
              {{ t('program_index.extra_settings')
              }}
            </el-menu-item>
          </el-sub-menu>

          <el-menu-item index="/setting_window/icons">
            <el-icon>
              <Picture />
            </el-icon>
            <span>{{ t('settings.icon_management') }}</span>
          </el-menu-item>

          <el-sub-menu index="/setting_window/search">
            <template #title>
              <el-icon>
                <List />
              </el-icon>
              <span>{{ t('settings.menu.other_search') }}</span>
            </template>
            <el-menu-item index="/setting_window/search/web">
              {{ t('settings.custom_web_search')
              }}
            </el-menu-item>
            <el-menu-item index="/setting_window/search/custom">
              {{ t('settings.custom_command_search')
              }}
            </el-menu-item>
            <el-menu-item index="/setting_window/search/builtin">
              {{ t('settings.builtin_command_settings')
              }}
            </el-menu-item>
            <el-menu-item index="/setting_window/search/everything">
              {{ t('everything.everything_search_settings')
              }}
            </el-menu-item>
            <el-menu-item index="/setting_window/search/bookmarks">
              {{ t('settings.search.bookmarks.title') }}
            </el-menu-item>
          </el-sub-menu>

          <el-menu-item index="/setting_window/config">
            <el-icon>
              <Connection />
            </el-icon>
            <span>{{ t('settings.menu.remote_management') }}</span>
          </el-menu-item>

          <el-menu-item index="/setting_window/shortcuts">
            <el-icon>
              <Key />
            </el-icon>
            <span>{{ t('settings.menu.shortcuts') }}</span>
          </el-menu-item>

          <el-menu-item index="/setting_window/about">
            <el-icon>
              <InfoFilled />
            </el-icon>
            <span>{{ t('settings.menu.about') }}</span>
          </el-menu-item>

          <el-menu-item
            v-if="isDebugMode"
            index="/setting_window/debug"
          >
            <el-icon>
              <Monitor />
            </el-icon>
            <span>{{ t('settings.debug_mode') }}</span>
          </el-menu-item>
        </el-menu>
      </el-scrollbar>

      <div class="footer-actions">
        <el-button
          type="primary"
          :loading="isSaving"
          :disabled="isSaveDisabled"
          class="save-btn"
          @click="saveConfig"
        >
          {{ t('settings.save_config') }}
        </el-button>
      </div>
    </div>

    <div class="content">
      <router-view v-slot="{ Component }">
        <transition
          name="fade"
          mode="out-in"
        >
          <component :is="Component" />
        </transition>
      </router-view>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref, onMounted, onUnmounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useRemoteConfigStore } from '../stores/remote_config'
import { storeToRefs } from 'pinia'
import { listen, UnlistenFn } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'
import { initializeLanguage } from '../i18n/index'
import {
    Setting,
    Brush,
    Search,
    List,
    Connection,
    Key,
    InfoFilled,
    Monitor,
    Picture,
} from '@element-plus/icons-vue'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()
const configStore = useRemoteConfigStore()
const { config } = storeToRefs(configStore)

const activeMenu = computed(() => route.path)
const isDebugMode = computed(() => config.value.app_config.is_debug_mode)
const isSaving = ref(false)
const is_dark = ref(false)

const applyTheme = (isDark: boolean) => {
  is_dark.value = isDark
  if (isDark) {
    document.documentElement.classList.add('dark')
  } else {
    document.documentElement.classList.remove('dark')
  }
}

const updateTheme = async () => {
  const mode = config.value.ui_config.frontend_theme_mode
  let isDark: boolean
  if (mode === 'dark') {
    isDark = true
    applyTheme(true)
  } else if (mode === 'light') {
    isDark = false
    applyTheme(false)
  } else {
    try {
      isDark = await invoke<boolean>('command_is_system_dark_mode')
      applyTheme(isDark)
    } catch (e) {
      console.error('Failed to get system theme from backend:', e)
      isDark = false
      applyTheme(false)
    }
  }
  // 同步设置窗口标题栏颜色
  const themeArg = mode === 'system' ? 'system' : (isDark ? 'dark' : 'light')
  invoke('set_window_theme', { windowLabel: 'setting_window', theme: themeArg }).catch(() => {})
}

watch(() => config.value.ui_config.frontend_theme_mode, () => {
  updateTheme()
})

const isDirty = computed(() => Object.keys(configStore.dirtyConfig).length > 0)
const restrictedPaths = [
    '/setting_window/config',
    '/setting_window/shortcuts',
    '/setting_window/about',
    '/setting_window/debug',
]
const isRestrictedPage = computed(() => restrictedPaths.some(path => route.path.startsWith(path)))

const isSaveDisabled = computed(() => {
    return !isDirty.value || isRestrictedPage.value
})

const handleMenuSelect = (index: string) => {
    if (restrictedPaths.some(path => index.startsWith(path))) {
        if (isDirty.value) {
            ElMessage.warning(t('settings.please_save_first'))
            return
        }
    }
    router.push(index)
}

const saveConfig = async () => {
    isSaving.value = true
    try {
        await configStore.syncConfig()
        ElMessage.success(t('settings.config_saved'))
    } catch (error) {
        ElMessage.error(t('settings.save_failed'))
    } finally {
        isSaving.value = false
    }
}

let unlisten: Array<UnlistenFn | null> = []

onMounted(async () => {
    await configStore.loadConfig()
    initializeLanguage(config.value.app_config.language)
    await updateTheme()

    unlisten.push(await listen('emit_update_setting_window_config', async () => {
        await configStore.loadConfig()
        initializeLanguage(config.value.app_config.language)
        await updateTheme()
    }))

    unlisten.push(await listen('system-theme-changed', (event) => {
        if (config.value.ui_config.frontend_theme_mode === 'system') {
            const theme = event.payload as string
            applyTheme(theme === 'dark')
            invoke('set_window_theme', { windowLabel: 'setting_window', theme }).catch(() => {})
        }
    }))
})

onUnmounted(() => {
    unlisten.forEach(fn => fn && fn())
    unlisten = []
    document.documentElement.classList.remove('dark')
})
</script>

<style>
/* Global dark mode overrides for Element Plus and settings pages */
html.dark {
  color-scheme: dark;
}

html.dark .el-form-item__label {
  color: #cfd3dc;
}

html.dark .el-divider__text {
  color: #a3a6ad;
}

html.dark .el-divider {
  border-color: #4c4d4f;
}

html.dark .page-title {
  color: #e5eaf3;
}

html.dark .el-icon {
  color: #a3a6ad;
}

html.dark .el-icon.el-question-icon {
  color: #73767a;
}

html.dark .el-text {
  color: #a3a6ad;
}

html.dark .el-form-item__content {
  color: #cfd3dc;
}

html.dark .settings-page .content-container {
  color: #cfd3dc;
}

/* Override hardcoded backgrounds in dark mode */
html.dark .el-input__wrapper,
html.dark .el-textarea__inner {
  background-color: #303133;
}

html.dark .el-table {
  --el-table-bg-color: #1d1e1f;
  --el-table-tr-bg-color: #1d1e1f;
  --el-table-header-bg-color: #262727;
  --el-table-row-hover-bg-color: #262727;
}

html.dark .el-card {
  --el-card-bg-color: #1d1e1f;
}

/* Override hardcoded inline backgrounds */
html.dark .el-tag--info {
  --el-tag-bg-color: #313233;
  --el-tag-text-color: #a3a6ad;
  --el-tag-border-color: #4c4d4f;
}

/* Override scoped page-level hardcoded backgrounds and colors */
html.dark .content-container,
html.dark .settings-page {
  color: #cfd3dc;
}

html.dark .path-list-section,
html.dark .path-detail-section {
  background-color: #1d1e1f !important;
  border-color: #4c4d4f !important;
}

html.dark .section-header {
  background-color: #262727 !important;
  border-color: #4c4d4f !important;
}

html.dark .path-item {
  border-color: #3a3a3a !important;
}

html.dark .path-item:hover {
  background-color: #303133 !important;
}

html.dark .key-display {
  background-color: #262727 !important;
  border-color: #4c4d4f !important;
  color: #a3a6ad !important;
}

html.dark .key-display:hover {
  background-color: #303133 !important;
  border-color: #5c5d5f !important;
}

html.dark .key-display.listening {
  background-color: #1d3a5c !important;
  border-color: #409eff !important;
  color: #409eff !important;
}

html.dark .key-display.captured {
  background-color: #2d2d2d !important;
  border-color: #4c4d4f !important;
}

html.dark .shortcut-label {
  color: #cfd3dc !important;
}

html.dark .about-card,
html.dark .changelog-box {
  background-color: #262727 !important;
}

html.dark .update-status.success {
  background-color: #1b2e1b !important;
  color: #67c23a !important;
}

html.dark .update-status.warning {
  background-color: #2e2518 !important;
  color: #e6a23c !important;
}

html.dark .update-status.error {
  background-color: #2e1b1b !important;
  color: #f56c6c !important;
}

html.dark .feature-item {
  background-color: #262727 !important;
}

html.dark .feature-item .el-icon {
  color: #409eff !important;
}

/* Fix el-menu text in sidebar */
html.dark .settings-menu .el-menu-item,
html.dark .settings-menu .el-sub-menu__title {
  color: #cfd3dc;
}

html.dark .settings-menu .el-menu-item:hover,
html.dark .settings-menu .el-sub-menu__title:hover {
  background-color: #303133;
}
</style>

<style scoped>
.settings-layout {
    display: flex;
    width: 100%;
    height: 100vh;
    background-color: #fff;
    overflow: hidden;
    transition: background-color 0.3s ease;
}

.settings-layout.dark {
    background-color: #1a1a1a;
}

.sidebar {
    width: 240px;
    background-color: #f5f7fa;
    display: flex;
    flex-direction: column;
    border-right: 1px solid #e6e6e6;
    flex-shrink: 0;
    transition: background-color 0.3s ease, border-color 0.3s ease;
}

.settings-layout.dark .sidebar {
    background-color: #252525;
    border-right-color: #3a3a3a;
}

.header {
    padding: 20px;
    display: flex;
    align-items: center;
    border-bottom: 1px solid #e6e6e6;
    transition: border-color 0.3s ease;
}

.settings-layout.dark .header {
    border-bottom-color: #3a3a3a;
}

.logo {
    width: 28px;
    height: 28px;
    margin-right: 10px;
}

.title {
    font-size: 18px;
    font-weight: 600;
    color: #303133;
    transition: color 0.3s ease;
}

.settings-layout.dark .title {
    color: #e0e0e0;
}

.settings-menu {
    border-right: none;
    background-color: transparent;
}

.footer-actions {
    padding: 16px;
    border-top: 1px solid #e6e6e6;
    display: flex;
    justify-content: center;
    background-color: #f5f7fa;
    transition: background-color 0.3s ease, border-color 0.3s ease;
}

.settings-layout.dark .footer-actions {
    background-color: #252525;
    border-top-color: #3a3a3a;
}

.save-btn {
    width: 100%;
}

.content {
    flex: 1;
    padding: 0;
    overflow: hidden;
    background-color: #fff;
    position: relative;
    transition: background-color 0.3s ease;
}

.settings-layout.dark .content {
    background-color: #1a1a1a;
}

/* Transition */
.fade-enter-active,
.fade-leave-active {
    transition: opacity 0.025s ease;
}

.fade-enter-from,
.fade-leave-to {
    opacity: 0;
}
</style>
