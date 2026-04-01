<script setup lang="ts">
import type { PluginInfo } from "@bridge/bridge";

const props = defineProps<{ items: PluginInfo[]; activeId?: string }>();
const emit = defineEmits<{ (e: "select", id: string): void }>();
</script>

<template>
  <li
    v-for="item in items"
    :key="item.id"
    @click.prevent="emit('select', item.id)"
    class="w-full cursor-pointer list-none overflow-y-auto"
  >
    <span
      :class="[
        'pl-page block w-full py-3 transition-colors duration-200',
        activeId === item.id
          ? 'bg-gray-100 text-gray-900' // 激活项：灰色背景，深色文字
          : 'bg-white text-gray-800 hover:bg-gray-50 active:bg-gray-200', // 默认白色背景，hover/active 渐变
      ]"
    >
      {{ item.name.toLocaleUpperCase() }}
    </span>
  </li>
</template>
