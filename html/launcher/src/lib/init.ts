export function setup() {
    setupDragRegion(window.MSG.DRAG);
    setupKeyboardShortcuts();
}

function setupDragRegion(msg: string) {
    document.addEventListener("DOMContentLoaded", () => {
        const dragElement = document.querySelector("[draggable-region]");
        let pressTimer: number | undefined = undefined;
        dragElement?.addEventListener("mousedown", e => {
            const target = e.target as Element;
            if (target?.closest('button, a, input, select, textarea, [contenteditable="true"]')) {
                return;
            }
            pressTimer = setTimeout(() => window.ipc.postMessage(msg), 500);
        });
        dragElement?.addEventListener("mouseup", () => {
            if (pressTimer) clearTimeout(pressTimer);
        });
        dragElement?.addEventListener("mouseleave", () => {
            if (pressTimer) clearTimeout(pressTimer);
        });
    });
}

function setupKeyboardShortcuts() {
    document.addEventListener("keydown", function (e) {
        // 禁用页面按键，交由rust监听
        if (e.ctrlKey) {
            e.preventDefault();
            return false;
        }
    });
    window.onKey = function (e: string) {};
}
