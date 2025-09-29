<script setup lang="ts">
import { computed } from "vue";
import { Table, TableBody, TableHeader, TableRow, TableCell, TableHead } from "@/components/ui/table";

// 定义更完善的类型
interface TableData {
    [key: string]: any;
}

interface Props {
    titles?: string[];
    data?: TableData[];
    value?: any[] | null | undefined;
    keyField?: string;
    emptyMessage?: string;
    showActionColumn?: boolean;
    clickableRows?: boolean;
    striped?: boolean;
    hoverEffect?: boolean;
    compact?: boolean;
    border?: boolean; // 新增：是否显示边框
    rounded?: boolean; // 新增：圆角效果
}

const props = withDefaults(defineProps<Props>(), {
    data: () => [],
    titles: () => [],
    value: () => [],
    emptyMessage: "No data available",
    keyField: "id",
    showActionColumn: false,
    clickableRows: false,
    striped: false,
    hoverEffect: true,
    compact: false,
    border: true,
    rounded: true,
});

const emit = defineEmits<{
    (e: "itemClick", item: any): void;
    (e: "actionClick", item: any, action?: string): void;
}>();

// 优化计算属性：统一处理数据源
const normalizedData = computed(() => {
    return props.data?.length > 0 ? props.data : props.value || [];
});

// 优化计算属性：统一处理标题
const normalizedTitles = computed(() => {
    if (props.titles?.length > 0) return props.titles;

    if (normalizedData.value.length > 0) {
        return Object.keys(normalizedData.value[0]);
    }

    return [];
});

// 优化计算属性：检查是否有数据
const hasData = computed(() => {
    return normalizedData.value.length > 0 && normalizedTitles.value.length > 0;
});

// 获取单元格值，支持嵌套属性
const getCellValue = (item: TableData, title: string) => {
    return title.split(".").reduce((obj, key) => {
        return obj && obj[key] !== undefined ? obj[key] : "";
    }, item);
};

// 处理行点击
const handleRowClick = (item: any) => {
    if (props.clickableRows) {
        emit("itemClick", item);
    }
};

// 处理操作点击
const handleActionClick = (item: any, action?: string) => {
    emit("actionClick", item, action);
};
</script>

<template>
    <div class="w-full overflow-x-auto" :class="{ 'rounded-lg': rounded, 'border border-gray-200': border }">
        <Table class="w-full border-collapse" :class="{ 'text-sm': compact, 'text-base': !compact }">
            <TableHeader>
                <TableRow class="bg-gray-50">
                    <TableHead v-for="title in normalizedTitles" :key="title" class="px-4 py-3 font-semibold text-gray-700 text-left border-b border-gray-200" :class="{ 'px-3 py-2': compact }">
                        {{ title }}
                    </TableHead>
                    <TableHead v-if="showActionColumn" class="px-4 font-semibold text-gray-700 py-3 text-center border-b border-gray-200 w-24" :class="{ 'px-3 py-2': compact }">
                        <slot name="actionHeader">
                            <span>Actions</span>
                        </slot>
                    </TableHead>
                </TableRow>
            </TableHeader>

            <TableBody>
                <TableRow v-if="!hasData">
                    <TableCell
                        :colspan="normalizedTitles.length + (showActionColumn ? 1 : 0)"
                        class="px-4 py-8 text-center text-gray-500 italic border-b border-gray-200"
                        :class="{ 'px-3 py-6': compact }">
                        {{ emptyMessage }}
                    </TableCell>
                </TableRow>

                <TableRow
                    v-for="(item, index) in normalizedData"
                    :key="item[keyField] || item.id || index"
                    class="transition-colors duration-150"
                    :class="{
                        'bg-gray-50/50': striped && index % 2 === 1,
                        'hover:bg-gray-100 cursor-pointer': hoverEffect && clickableRows,
                        'hover:bg-gray-50': hoverEffect && !clickableRows,
                        'cursor-pointer': clickableRows,
                    }"
                    @click="handleRowClick(item)">
                    <TableCell v-for="title in normalizedTitles" :key="title" class="px-4 py-3 text-gray-900 border-b border-gray-200" :class="{ 'px-3 py-2': compact, truncate: compact }">
                        <!-- 支持自定义单元格内容 -->
                        <slot :name="`cell-${title}`" :value="getCellValue(item, title)" :item="item">
                            <span class="truncate block">{{ getCellValue(item, title) }}</span>
                        </slot>
                    </TableCell>

                    <TableCell v-if="showActionColumn" class="px-4 py-3 text-center border-b border-gray-200" :class="{ 'px-3 py-2': compact }" @click.stop="handleActionClick(item)">
                        <slot name="actionCell" :item="item" :onActionClick="handleActionClick">
                            <button
                                class="inline-flex items-center justify-center w-8 h-8 rounded-md border border-gray-300 bg-white text-gray-600 hover:bg-gray-50 hover:text-gray-700 transition-colors duration-150"
                                @click.stop="handleActionClick(item, 'default')">
                                <span class="text-lg leading-none">⋯</span>
                            </button>
                        </slot>
                    </TableCell>
                </TableRow>
            </TableBody>
        </Table>
    </div>
</template>

<style scoped></style>
