import { ref, computed, type Ref } from "vue";

// 请求状态接口
export interface RequestState<T = any> {
    loading: boolean;
    error: Error | null;
    data: T | null;
    isEmpty: boolean;
}

// 回调函数接口
export interface RequestCallbacks<T> {
    onLoading?: () => void;
    onError?: (error: Error) => void;
    onEmpty?: () => void;
    onData?: (data: T) => void;
}

// 组件 Props 接口
export interface RequestStatusProps<T = any> {
    state: RequestState<T>;
    showSuccess?: boolean;
    successDuration?: number;
    containerClass?: string;
    loadingText?: string;
    errorText?: string;
    emptyText?: string;
}

// 组合式函数
export function useRequestState<T = any>() {
    const state = ref<RequestState<T>>({
        loading: false,
        error: null,
        data: null,
        isEmpty: false,
    });

    const isEmpty = computed(() => {
        const data = state.value.data;
        if (!data) return true;
        if (Array.isArray(data) && data.length === 0) return true;
        if (typeof data === "object" && Object.keys(data).length === 0) return true;
        return false;
    });

    async function withState(request: () => Promise<T>, callbacks: RequestCallbacks<T> = {}): Promise<T> {
        const { onLoading, onError, onEmpty, onData } = callbacks;

        // 设置加载状态
        state.value.loading = true;
        state.value.error = null;
        state.value.data = null;
        state.value.isEmpty = false;

        if (onLoading) {
            onLoading();
        }

        try {
            const response = await request();
            state.value.data = response as any;
            state.value.isEmpty = isEmpty.value;
            state.value.loading = false;

            if (state.value.isEmpty) {
                onEmpty?.();
            } else {
                onData?.(response);
            }

            return response;
        } catch (err) {
            state.value.error = err as Error;
            state.value.loading = false;
            onError?.(err as Error);
            throw err;
        }
    }

    function reset() {
        state.value.loading = false;
        state.value.error = null;
        state.value.data = null;
        state.value.isEmpty = false;
    }

    return {
        state: state as Ref<RequestState<T>>,
        withState,
        reset,
        isEmpty,
    };
}

// 默认导出组合式函数
export default useRequestState;
export { default as ReqStatus } from "./UseReqState.vue";
