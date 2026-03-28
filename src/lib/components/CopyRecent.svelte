<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import type { CopyResult } from '$lib/types';

  export let sessionId: string = '';

  let destPath = '';
  let copying = false;
  let result: CopyResult | null = null;
  let error = '';

  async function pickDest() {
    const selected = await open({ directory: true, multiple: false });
    if (selected && typeof selected === 'string') destPath = selected;
  }

  async function doCopy() {
    if (!destPath || !sessionId) return;
    copying = true;
    error = '';
    result = null;
    try {
      result = await invoke<CopyResult>('cmd_copy_recent_to_dest', {
        destination: destPath,
        sessionId
      });
    } catch (e: any) {
      error = String(e);
    }
    copying = false;
  }
</script>

<div class="copy-root">
  <h3 class="section-title">Copy Recent Session to Destination</h3>
  <p class="hint">Copies all renamed files from the last session to a folder (e.g. USB drive).</p>

  <div class="row">
    <input class="path-input" readonly value={destPath} placeholder="Select destination folder…" />
    <button class="btn btn-secondary" on:click={pickDest}>Browse</button>
  </div>

  <button
    class="btn btn-primary"
    disabled={!destPath || !sessionId || copying}
    on:click={doCopy}
  >
    {copying ? 'Copying…' : 'Copy Files'}
  </button>

  {#if result}
    <div class="result-box">
      <span class="res-item green">✓ {result.copied} copied</span>
      <span class="res-item gray">⊘ {result.skipped} skipped (duplicates)</span>
      {#if result.errors.length}
        <div class="err-list">
          {#each result.errors as e}<div class="err-item">⚠ {e}</div>{/each}
        </div>
      {/if}
    </div>
  {/if}

  {#if error}
    <div class="err-item">{error}</div>
  {/if}
</div>

<style>
  .copy-root { display: flex; flex-direction: column; gap: 14px; max-width: 600px; }
  .section-title { font-size: 1rem; font-weight: 700; color: var(--color-text); margin: 0; }
  .hint { font-size: 0.8rem; color: var(--color-muted); margin: 0; }
  .row { display: flex; gap: 8px; }
  .path-input {
    flex: 1; background: var(--color-input-bg); border: 1px solid var(--color-border);
    color: var(--color-muted); border-radius: 6px; padding: 6px 10px; font-size: 0.82rem;
  }
  .btn {
    padding: 7px 16px; border-radius: 6px; font-size: 0.82rem;
    font-weight: 600; cursor: pointer; border: none; transition: opacity 0.15s;
  }
  .btn:disabled { opacity: 0.4; cursor: not-allowed; }
  .btn-primary { background: var(--color-accent); color: #000; }
  .btn-secondary { background: var(--color-surface2); color: var(--color-text); border: 1px solid var(--color-border); }
  .result-box { display: flex; flex-direction: column; gap: 6px; padding: 12px; background: var(--color-surface2); border-radius: 8px; }
  .res-item { font-size: 0.8rem; }
  .green { color: #4ade80; }
  .gray { color: var(--color-muted); }
  .err-list { display: flex; flex-direction: column; gap: 4px; }
  .err-item { font-size: 0.78rem; color: #f87171; }
</style>