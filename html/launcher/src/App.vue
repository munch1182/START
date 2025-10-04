<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
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

onMounted(async () => {
    ph.value = "现在是" + formatDate(new Date(), "YYYY年MM月DD日HH点mm分");
    await refresh();
});

watch(input, refresh);

async function refresh(q: string | undefined = undefined) {
    state.value = StateValue.Loading;
    result.value = (await get("/search/s", { q: q ?? "" })) ?? [];
    if (result.value == undefined) {
        state.value = StateValue.Error;
    } else {
        state.value = result.value && result.value.length > 0 ? StateValue.Data : StateValue.Empty;
    }
}
</script>

<template>
    <main draggable-region :class="['flex flex-col items-center h-main bg-[#EFEFEF] rounded-md', isWeb ? 'm-32 rounded-lg' : '']">
        <Search v-model="input" :placeholder="ph"></Search>
        <hr class="w-full h-[0.2px] bg-[#A3A3A3]" />
        <State :state="state" class="w-full h-scroll">
            <ScrollArea class="h-scroll">
                <ul class="rounded-md">
                    <li v-for="r in result" class="flex flex-row items-center w-full h-item px-4 py-2 hover:bg-[#E6E6E6]">
                        <img :src="r.icon" />
                        <span>{{ r.name }}</span>
                        <span class="ml-auto text-sm">{{ r.type }}</span>
                    </li>
                </ul>
            </ScrollArea>
        </State>
    </main>
</template>
