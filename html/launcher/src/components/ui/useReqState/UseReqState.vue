<template>
  <div class="relative" :class="containerClass">
    <!-- 加载状态 -->
    <div v-if="state.loading"
      class="absolute inset-0 flex flex-col items-center justify-center bg-white bg-opacity-90 z-10 rounded-lg p-6">
      <slot name="loading" :state="state">
        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mb-4"></div>
        <h3 class="text-lg font-semibold text-gray-700">加载中...</h3>
        <p class="text-gray-500 mt-2">{{ loadingText }}</p>
      </slot>
    </div>

    <!-- 错误状态 -->
    <div v-else-if="state.error"
      class="absolute inset-0 flex flex-col items-center justify-center bg-white bg-opacity-90 z-10 rounded-lg p-6">
      <slot name="error" :error="state.error">
        <div class="text-red-500 mb-4">
          <svg class="w-12 h-12 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
          </svg>
        </div>
        <h3 class="text-lg font-semibold text-gray-700">加载失败</h3>
        <p class="text-gray-600 mt-2 text-center">{{ state.error?.message || errorText }}</p>
        <button @click="$emit('retry')"
          class="mt-4 px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors">
          重试
        </button>
      </slot>
    </div>

    <!-- 空状态 -->
    <div v-else-if="state.isEmpty"
      class="absolute inset-0 flex flex-col items-center justify-center bg-white bg-opacity-90 z-10 rounded-lg p-6">
      <slot name="empty" :state="state">
        <div class="text-gray-400 mb-4">
          <svg class="w-12 h-12 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2M4 13h2"></path>
          </svg>
        </div>
        <h3 class="text-lg font-semibold text-gray-700">暂无数据</h3>
        <p class="text-gray-500 mt-2">{{ emptyText }}</p>
      </slot>
    </div>

    <!-- 成功状态 -->
    <div v-else-if="showSuccess"
      class="absolute inset-0 flex flex-col items-center justify-center bg-white bg-opacity-90 z-10 rounded-lg p-6">
      <slot name="success" :data="state.data">
        <div class="text-green-500 mb-4">
          <svg class="w-12 h-12 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"></path>
          </svg>
        </div>
        <h3 class="text-lg font-semibold text-gray-700">加载成功</h3>
        <p class="text-gray-500 mt-2">已成功获取数据</p>
      </slot>
    </div>

    <!-- 默认内容 -->
    <slot v-else :data="state.data" :state="state" />
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import type { RequestStatusProps } from './index'

// 定义 Props
const props = withDefaults(defineProps<RequestStatusProps>(), {
  showSuccess: false,
  successDuration: 2000,
  containerClass: '',
  loadingText: '请稍候，数据正在加载',
  errorText: '请求过程中发生错误',
  emptyText: '当前没有可显示的内容'
})

// 定义事件
const emit = defineEmits<{ retry: [] }>()

// 响应式数据
const showSuccess = ref(false)

// 监听数据变化
watch(
  () => props.state.data,
  (newData) => {
    if (newData && !props.state.loading && !props.state.error && !props.state.isEmpty) {
      if (props.showSuccess) {
        showSuccess.value = true
        setTimeout(() => {
          showSuccess.value = false
        }, props.successDuration)
      }
    }
  },
  { immediate: true }
)
</script>