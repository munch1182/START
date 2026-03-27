<script setup>
import { defineProps, defineEmits } from "vue";

defineProps({
  /**
   * 是否禁用按钮
   */
  disabled: {
    type: Boolean,
    default: false,
  },
});
</script>

<template>
  <button
    class="inline-flex items-center justify-center bg-transparent transition-transform duration-200 ease-in-out [clip-path:circle(50%_at_50%_50%)] hover:bg-black/10 focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:outline-none active:scale-[0.96] disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-60 dark:hover:bg-white/20"
    :disabled="disabled"
    v-bind="$attrs"
  >
    <span class="inline-flex items-center justify-center leading-none">
      <slot />
    </span>
  </button>
</template>

<style scoped>
button :deep(img),
button :deep(svg) {
  display: block;
  max-width: 100%;
  height: auto;
}
</style>
