<script setup lang="ts">
import { RiCloseLine, RiSubtractLine } from "@remixicon/vue";
import IconButton from "./widgets/IconButton.vue";
</script>

<template>
  <div class="flex w-full justify-end border-b border-gray-200">
    <IconButton class="m-1 size-8" data-command="Minimize">
      <RiSubtractLine size="24" />
    </IconButton>
    <IconButton class="m-1 size-8 hover:text-red-600" data-command="Close">
      <RiCloseLine size="24" />
    </IconButton>
  </div>
</template>

<style scoped></style>
