<script setup lang="ts">
import { ref, onMounted, watch, onUnmounted, nextTick } from "vue";
import { formatDate } from "@vueuse/core";
import { Search } from "@/components/ui/search";
import { State, StateValue, type StateType } from "@/components/ui/state";
import { ScrollArea } from "@/components/ui/scroll-area";
import { get } from "./lib/net";

interface Result {
    id?: string;
    icon?: string;
    name?: string;
    type?: string;
}

const isWeb = window.IS_WEB;
const result = ref<Array<Result> | undefined>([]);
const input = ref("");
const ph = ref("");
const state = ref<StateType>(StateValue.Loading);
const selectedIndex = ref(-1);
const isCtrlPressed = ref(false);

onMounted(async () => {
    document.addEventListener("keydown", handleKeyDown);
    document.addEventListener("keyup", handleKeyUp);
    ph.value = "现在是" + formatDate(new Date(), "YYYY年MM月DD日HH点mm分");
    await refresh();
    await nextTick();
});
onUnmounted(() => {
    document.removeEventListener("keydown", handleKeyDown);
    document.removeEventListener("keyup", handleKeyUp);
});

function handleKeyUp(e: KeyboardEvent) {
    if (e.key === "Alt") {
        isCtrlPressed.value = false;
    }
}

function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Alt") {
        isCtrlPressed.value = true;
        return;
    }
    const target = e.target as HTMLElement;
    if (target.tagName === "input" || !result.value || result.value.length === 0) return;
    switch (e.key) {
        case "ArrowUp":
            e.preventDefault();
            navigate(-1);
            break;
        case "ArrowDown":
            e.preventDefault();
            navigate(1);
            break;
        case "Enter":
            e.preventDefault();
            break;
        default:
            // 处理数字键1-9
            if (/^[1-9]$/.test(e.key)) {
                if (isCtrlPressed.value) {
                    e.preventDefault();
                    const newIndex = parseInt(e.key) - 1;
                    selectedIndex.value = newIndex >= result.value.length ? result.value.length - 1 : newIndex;
                }
            }
            break;
    }
}

function navigate(num: number) {
    const len = result.value?.length;
    if (len == undefined) return;

    if (0 < num && selectedIndex.value >= len - 1) {
        selectedIndex.value = 0;
    } else if (0 > num && selectedIndex.value <= 0) {
        selectedIndex.value = len - 1;
    } else {
        selectedIndex.value += num;
    }
}

watch(input, refresh);

async function refresh(q: string | undefined = undefined) {
    state.value = StateValue.Loading;
    result.value = (await get("/search/s", { q: q ?? "" })) ?? [];
    if (isWeb) {
        result.value = [
            { id: "web", name: "Google", type: "搜索" },
            { id: "web", name: "Baidu", type: "搜索" },
            { id: "web", name: "chrome", type: "应用" },
            { id: "web", name: "文件管理器", type: "应用" },
            { id: "web", name: "VS Code", type: "应用" },
        ];
    }
    state.value = result.value && result.value.length > 0 ? StateValue.Data : StateValue.Empty;
}
</script>

<template>
    <main draggable-region :class="['flex flex-col items-center h-main bg-[#EFEFEF]', isWeb ? 'm-32 rounded-lg' : '']">
        <Search ref="inputref" v-model="input" :placeholder="ph"></Search>
        <hr class="w-full h-[0.2px] bg-[#A3A3A3]" />
        <State :state="state" class="w-full h-scroll">
            <ScrollArea class="h-scroll">
                <ul>
                    <li
                        v-for="(r, i) in result"
                        :key="r.name"
                        :class="['flex flex-row items-center w-full h-item px-4 py-2 hover:bg-[#E6E6E6] cursor-pointer transition-colors', selectedIndex === i ? 'bg-[#E6E6E6]' : '']">
                        <span v-if="isCtrlPressed && i < 9" class="w-6 h-6 flex items-center justify-center">{{ i + 1 }}:</span>
                        <img :src="r.icon" />
                        <span>{{ r.name }}</span>
                        <span class="ml-auto text-sm">{{ r.type }}</span>
                    </li>
                </ul>
            </ScrollArea>
        </State>
    </main>
</template>
