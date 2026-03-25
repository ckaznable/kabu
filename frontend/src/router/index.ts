import { createRouter, createWebHistory } from 'vue-router'
import OverviewView from '../views/OverviewView.vue'
import SettingsView from '../views/SettingsView.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', name: 'overview', component: OverviewView },
    { path: '/settings', name: 'settings', component: SettingsView },
  ],
})

export default router
