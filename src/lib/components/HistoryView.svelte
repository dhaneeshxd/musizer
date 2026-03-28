<script lang="ts">
  import type { RenameRecord, } from '$lib/types';
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';


  let records: RenameRecord[] = [];
  let error = "";


  onMount(loadHistory);

  function statusClass(s: string) {
    if (s === 'Renamed') return 'pill-green';
    if (s === 'Moved') return 'pill-red';
    return 'pill-gray';
  }

  function basename(p: string) {
    return p.split(/[/\\]/).pop() ?? p;
  }

  function formatTime(iso: string) {
    try {
      return new Date(iso).toLocaleString();
    } catch {
      return iso;
    }
  }
    async function deleteHistory() {
    try {
        const deleted = await invoke<number>('cmd_delete_all_history');
        console.log('Deleted rows:', deleted);
        records = [];
    } catch (e: any) {
      error = String(e);
    }
  }

  async function loadHistory() {
  records = await invoke('cmd_get_recent_history', { limit: 200 });
}

</script>

<div class="history-root">
    <button
    class="btn btn-danger"
    onclick={deleteHistory}
  >
    Delete All History
  </button>
  {#if records.length === 0}
    <div class="empty">No history records yet.</div>
  {:else}
    <div class="table-wrap">
      <table>
        <thead>
          <tr>
            <th>Status</th>
            <th>Original</th>
            <th>New Path</th>
            <th>Bitrate</th>
            <th>Time</th>
          </tr>
        </thead>
        <tbody>
          {#each records as r}
            <tr>
              <td><span class="pill {statusClass(r.status)}">{r.status}</span></td>
              <td class="path-cell" title={r.original_path}>{basename(r.original_path)}</td>
              <td class="path-cell" title={r.new_path}>{basename(r.new_path)}</td>
              <td class="mono">{r.bitrate} BIT</td>
              <td class="mono small">{formatTime(r.timestamp)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<style>
  .history-root { width: 100%; }
  .empty { color: var(--color-muted); text-align: center; padding: 32px; }
  .table-wrap { overflow-x: auto; border-radius: 10px; border: 1px solid var(--color-border); }
  table { width: 100%; border-collapse: collapse; font-size: 0.82rem; }
  thead th {
    background: var(--color-thead); color: var(--color-muted);
    font-weight: 600; text-transform: uppercase; letter-spacing: 0.06em;
    font-size: 0.7rem; padding: 10px 12px; text-align: left;
    border-bottom: 1px solid var(--color-border);
  }
  tbody tr { border-bottom: 1px solid var(--color-border); }
  tbody tr:last-child { border-bottom: none; }
  td { padding: 8px 12px; vertical-align: middle; }
  .pill { display: inline-block; padding: 2px 8px; border-radius: 99px; font-size: 0.7rem; font-weight: 600; }
  .pill-green { background: rgba(74,222,128,0.2); color: #4ade80; }
  .pill-red { background: rgba(248,113,113,0.2); color: #f87171; }
  .pill-gray { background: rgba(156,163,175,0.2); color: #9ca3af; }
  .path-cell { max-width: 220px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--color-muted); }
  .mono { font-family: 'JetBrains Mono', monospace; font-size: 0.75rem; }
  .small { font-size: 0.7rem; }
.btn {
  padding: 7px 16px;
  border-radius: 6px;
  font-size: 0.82rem;
  font-weight: 600;
  cursor: pointer;
  border: none;
  margin-bottom: 20px;

  transition: all 0.15s ease;
  transform: translateY(0);
}

/* Hover effect */
.btn:hover {
  opacity: 0.9;
  transform: translateY(-1px);
}

/* Click / press effect */
.btn:active {
  transform: translateY(1px) scale(0.98);
  opacity: 0.85;
}

/* Primary button */
.btn-primary {
  background: var(--color-accent);
  color: hsl(0, 100%, 1%);
  box-shadow: 0 2px 6px rgba(0,0,0,0.2);
}

/* 🔥 Delete / danger button */
.btn-danger {
  background: #ef4444; /* red-500 */
  color: white;
  box-shadow: 0 2px 8px rgba(239, 68, 68, 0.4);
}

/* Danger hover glow */
.btn-danger:hover {
  background: #dc2626; /* darker red */
  box-shadow: 0 4px 12px rgba(239, 68, 68, 0.6);
}

/* Optional: focus ring */
.btn:focus-visible {
  outline: none;
  box-shadow: 0 0 0 2px white, 0 0 0 4px rgba(59,130,246,0.6);
}
</style>