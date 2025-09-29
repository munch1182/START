import { createWebHistory, createRouter } from "vue-router";
import Search from "@/components/Search.vue";

export default createRouter({
    history: createWebHistory(),
    routes: [
        {
            path: "/",
            redirect: "/debug"
        },
        {
            path: "/home",
            component: Search,
        },
        // {
        //     path: "/debug",
        //     component: () => import("@/components/debug/DebugOp.vue"),
        //     children: [
        //         {
        //             path: "adb",
        //             component: () => import("@/components/debug/DebugAdb.vue")
        //         }
        //     ]
        // }
    ]
});