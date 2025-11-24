<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import { onMount, onDestroy } from "svelte";

    export let isOpen = false;
    export let title = "Confirm Action";
    export let message = "Are you sure you want to proceed?";
    export let confirmLabel = "Confirm";
    export let cancelLabel = "Cancel";
    export let isDanger = false;

    const dispatch = createEventDispatcher();

    // Lock body scroll when modal is open
    $: if (isOpen) {
        document.body.style.overflow = "hidden";
    } else {
        document.body.style.overflow = "";
    }

    onDestroy(() => {
        document.body.style.overflow = "";
    });

    function handleConfirm() {
        dispatch("confirm");
        close();
    }

    function handleCancel() {
        dispatch("cancel");
        close();
    }

    function close() {
        isOpen = false;
    }
</script>

{#if isOpen}
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div class="modal-backdrop" on:click={handleCancel}>
        <div
            class="modal-content"
            on:click|stopPropagation
            role="dialog"
            aria-modal="true"
            tabindex="-1"
        >
            <header class="modal-header">
                <h3>{title}</h3>
            </header>

            <div class="modal-body">
                <p>{message}</p>
            </div>

            <footer class="modal-footer">
                <button class="btn-text" on:click={handleCancel}
                    >{cancelLabel}</button
                >
                <button
                    class="btn-primary {isDanger ? 'danger' : ''}"
                    on:click={handleConfirm}
                >
                    {confirmLabel}
                </button>
            </footer>
        </div>
    </div>
{/if}

<style>
    .modal-backdrop {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: rgba(0, 0, 0, 0.6);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 9999;
        animation: fadeIn 0.15s ease-out;
        padding: 16px;
    }

    @keyframes fadeIn {
        from {
            opacity: 0;
        }
        to {
            opacity: 1;
        }
    }

    .modal-content {
        background: rgb(var(--surface-rgb));
        border: 1px solid var(--border);
        border-radius: var(--radius);
        width: 100%;
        max-width: 450px;
        display: flex;
        flex-direction: column;
        box-shadow: var(--shadow-lg);
        animation: slideUp 0.2s ease-out;
    }

    @media (max-width: 500px) {
        .modal-content {
            max-width: 100%;
            border-radius: var(--radius-sm);
        }

        .modal-header,
        .modal-body,
        .modal-footer {
            padding: 16px;
        }
    }

    @keyframes slideUp {
        from {
            opacity: 0;
            transform: translateY(20px) scale(0.98);
        }
        to {
            opacity: 1;
            transform: translateY(0) scale(1);
        }
    }

    .modal-header {
        padding: 20px 24px;
        border-bottom: 1px solid var(--border);
        background: rgb(var(--surface-rgb));
        border-radius: var(--radius) var(--radius) 0 0;
    }

    .modal-header h3 {
        margin: 0;
        font-size: 1.1rem;
        font-weight: 600;
        color: var(--text);
    }

    .modal-body {
        padding: 24px;
        background: rgb(var(--surface-rgb));
    }

    .modal-body p {
        margin: 0;
        color: var(--text-secondary);
        line-height: 1.6;
    }

    .modal-footer {
        padding: 20px 24px;
        border-top: 1px solid var(--border);
        display: flex;
        justify-content: flex-end;
        gap: 12px;
        background: rgb(var(--surface-muted-rgb));
        border-radius: 0 0 var(--radius) var(--radius);
    }

    .btn-text {
        background: none;
        border: none;
        padding: 8px 16px;
        cursor: pointer;
        color: var(--text-secondary);
        font-weight: 500;
        transition: color 0.2s;
        border-radius: var(--radius-sm);
    }

    .btn-text:hover {
        color: var(--text);
        background: var(--bg-hover);
    }

    .btn-primary {
        background: var(--accent);
        color: white;
        border: none;
        padding: 8px 24px;
        border-radius: var(--radius-sm);
        font-weight: 600;
        cursor: pointer;
        transition: all 0.2s;
        box-shadow: var(--shadow-sm);
    }

    .btn-primary:hover {
        opacity: 0.9;
        transform: translateY(-1px);
        box-shadow: var(--shadow);
    }

    .btn-primary.danger {
        background: var(--danger);
    }

    .btn-primary.danger:hover {
        background: color-mix(in srgb, var(--danger) 90%, black);
    }
</style>
