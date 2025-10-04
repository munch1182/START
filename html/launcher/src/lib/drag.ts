export function setupDragRegion(msg: string) {
    document.addEventListener("DOMContentLoaded", () => {
        let dragElement = document.querySelector("[draggable-region]");
        dragElement?.addEventListener("mousedown", e => {
            const target = e.target as Element;
            if (target?.closest('button, a, input, select, textarea, [contenteditable="true"]')) {
                return;
            }
            window.ipc.postMessage(msg);
        });
    });
}

export function setupKeyboardShortcuts() {
    document.addEventListener("keydown", function (e) {
        // 禁用页面按键，交由rust监听
        if (e.ctrlKey) {
            e.preventDefault();
            return false;
        }
    });
}
