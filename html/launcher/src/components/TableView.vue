<script setup lang="ts">
import { defineProps, computed } from 'vue'
import {
    Table,
    TableBody,
    TableHeader,
    TableRow,
    TableCell,
    TableHead
} from '@/components/ui/table'

// 定义更通用的数据类型
interface TableData {
    [key: string]: any
}

interface Props {
    titles: string[]
    data?: TableData[]
    keyField?: string // 用于行的唯一标识
    emptyMessage?: string
    showActionColumn?: boolean  // 是否显示操作列
}

const props = withDefaults(defineProps<Props>(), {
    data: () => [],
    emptyMessage: 'No data available',
    keyField: 'id',
    showActionColumn: false
})

const emit = defineEmits<{
    (e: 'itemClick', item: TableData): void
}>()

// 计算属性：检查是否有数据
const hasData = computed(() =>
    props.data && props.data.length > 0 && props.titles && props.titles.length > 0
)

// 获取单元格的值
const getCellValue = (item: TableData, title: string) => {
    return item[title] !== undefined ? item[title] : ''
}
</script>

<template>
    <Table v-if="hasData" class="border border-gray-300">
        <TableHeader>
            <TableRow class="flex flex-1 pl-2 pr-2">
                <TableHead v-for="title in props.titles" :key="title" class="item">
                    {{ title }}
                </TableHead>
                <TableHead v-if="props.showActionColumn" class="item">
                    <!-- 操作列标题插槽 -->
                    <slot name="actionHeader">Op</slot>
                </TableHead>
            </TableRow>
        </TableHeader>
        <TableBody>
            <TableRow v-for="item in props.data" :key="item[keyField] || item.id || Math.random()"
                @click=" emit('itemClick', item)" class="flex flex-1 pl-2 pr-2">
                <TableCell v-for="title in props.titles" :key="title" class="item">
                    {{ getCellValue(item, title) }}
                </TableCell>
                <TableCell v-if="props.showActionColumn" class="item" @click.stop>
                    <!-- 操作列内容插槽 -->
                    <slot name="actionCell" :item="item"></slot>
                </TableCell>
            </TableRow>
        </TableBody>
    </Table>
</template>

<style scoped>
.item {
    @apply flex flex-1 text-center justify-center items-center text-ellipsis overflow-hidden;
}
</style>