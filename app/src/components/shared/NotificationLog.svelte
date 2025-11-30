<script lang="ts">
  import {
    notifications,
    dismissNotification,
    type NotificationEntry,
  } from "../lib/notifications";

  let entries: NotificationEntry[] = [];
  const unsubscribe = notifications.subscribe((value) => {
    entries = value;
  });

  import { onDestroy } from "svelte";
  onDestroy(() => unsubscribe());
</script>

<section class="notifications">
  <h2>Events & Errors</h2>
  {#if entries.length === 0}
    <p class="placeholder">No messages yet.</p>
  {:else}
    <ul>
      {#each entries as entry}
        <li class={entry.level}>
          <div>
            <span class="badge"
              >{entry.level === "error" ? "Error" : "Info"}</span
            >
            <span class="message">{entry.message}</span>
          </div>
          <button class="dismiss" on:click={() => dismissNotification(entry.id)}
            >×</button
          >
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .notifications {
    border: 1px solid #e2e8f0;
    border-radius: 8px;
    padding: 12px;
    background: #f8fafc;
    margin-top: 12px;
  }

  h2 {
    margin: 0 0 8px 0;
    font-size: 16px;
  }

  ul {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-height: 220px;
    overflow-y: auto;
  }

  li {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border: 1px solid #e2e8f0;
    border-radius: 6px;
    padding: 8px 10px;
    background: #ffffff;
    gap: 8px;
  }

  li.info {
    border-color: #cbd5e1;
  }

  li.error {
    border-color: #fca5a5;
    background: #fff1f2;
  }

  .badge {
    font-size: 12px;
    font-weight: 700;
    color: #334155;
    margin-right: 8px;
  }

  .message {
    font-size: 13px;
  }

  .dismiss {
    border: none;
    background: transparent;
    cursor: pointer;
    font-size: 16px;
    line-height: 1;
  }

  .placeholder {
    margin: 0;
    color: #94a3b8;
    font-size: 13px;
  }
</style>
