<script setup lang="ts">
import { StateValue, type StateType } from ".";
import { Button } from "../button";
import Loading from "../loading/Loading.vue";

const props = defineProps<{
    state: StateType;
}>();
</script>

<template>
    <div class="relative">
        <div v-if="props.state === StateValue.Loading" class="mask">
            <slot name="loading">
                <div class="flex flex-col items-center gap-1">
                    <Loading class="w-10 h-10"></Loading>
                    <span class="text-sm">加载中...</span>
                </div>
            </slot>
        </div>
        <div v-else-if="props.state === StateValue.Empty" class="mask">
            <slot name="empty">
                <span class="text-sm">暂无数据</span>
            </slot>
        </div>
        <div v-else-if="props.state === StateValue.Error" class="mask">
            <slot name="error">
                <span class="text-sm">发生了错误，</span>
                <Button variant="text" class="text-sm" @click="$emit('retry')">请重试</Button>
            </slot>
        </div>
        <slot v-else></slot>
    </div>
</template>

<style>
.mask {
    @apply w-full h-full flex items-center justify-center;
}
</style>
