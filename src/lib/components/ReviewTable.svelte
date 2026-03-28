<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { scannedFiles } from '$lib/stores';
  import type { AudioFile } from '$lib/types';

  export let files: AudioFile[] = [];

  // Inline edit state
  let editing: { idx: number; field: string } | null = null;
  let editValue = '';

  function rowClass(f: AudioFile): string {
    if (f.status === 'duplicate') return 'row-red';
    const hasUnknown =
      f.artist === 'Unknown' || f.album === 'Unknown' || f.title === f.file_stem;
    if (hasUnknown) return 'row-yellow';
    return 'row-green';
  }

  function rowLabel(f: AudioFile): string {
    if (f.status === 'duplicate') return 'Duplicate';
    const hasUnknown =
      f.artist === 'Unknown' || f.album === 'Unknown' || f.title === f.file_stem;
    if (hasUnknown) return 'Unknown metadata';
    return 'Ready';
  }

  function startEdit(idx: number, field: string, current: string) {
    editing = { idx, field };
    editValue = current;
  }

  async function commitEdit() {
    if (!editing) return;
    const { idx, field } = editing;
    try {
      const updated: AudioFile = await invoke('cmd_update_file_metadata', {
        file: files[idx],
        field,
        value: editValue
      });
      files[idx] = updated;
      scannedFiles.set([...files]);
    } catch (e) {
      console.error(e);
    }
    editing = null;
  }

  function cancelEdit() {
    editing = null;
  }

  function targetName(f: AudioFile): string {
    const sanitize = (s: string) =>
      s.replace(/[/\\:*?"<>|]/g, '_').trim();
    return `${sanitize(f.title)} - ${sanitize(f.album)} - ${f.bitrate}BIT.${f.extension}`;
  }

  // Counters
  $: greenCount = files.filter(
    (f) =>
      f.status !== 'duplicate' &&
      f.artist !== 'Unknown' &&
      f.album !== 'Unknown' &&
      f.title !== f.file_stem
  ).length;
  $: yellowCount = files.filter(
    (f) =>
      f.status !== 'duplicate' &&
      (f.artist === 'Unknown' || f.album === 'Unknown' || f.title === f.file_stem)
  ).length;
  $: redCount = files.filter((f) => f.status === 'duplicate').length;
</script>

<div class="review-root">
  <!-- Summary badges -->
  <div class="badges">
    <span class="badge badge-green">✓ {greenCount} Ready</span>
    <span class="badge badge-yellow">⚠ {yellowCount} Unknown metadata</span>
    <span class="badge badge-red">✕ {redCount} Duplicates</span>
  </div>

  <div class="table-wrap">
    <table>
      <thead>
        <tr>
          <th>Status</th>
          <th>Title</th>
          <th>Artist</th>
          <th>Album</th>
          <th>Bitrate</th>
          <th>New Name Preview</th>
        </tr>
      </thead>
      <tbody>
        {#each files as file, i}
          <tr class={rowClass(file)}>
            <!-- Status badge -->
            <td>
              <span class="status-pill pill-{file.status === 'duplicate' ? 'red' : (file.artist === 'Unknown' || file.album === 'Unknown') ? 'yellow' : 'green'}">
                {rowLabel(file)}
              </span>
            </td>

            <!-- Title (editable if yellow/unknown) -->
            <td>
              {#if editing && editing.idx === i && editing.field === 'title'}
                <!-- svelte-ignore a11y_autofocus -->
                <input
                  class="inline-input"
                  bind:value={editValue}
                  on:blur={commitEdit}
                  on:keydown={(e) => { if (e.key === 'Enter') commitEdit(); if (e.key === 'Escape') cancelEdit(); }}
                  autofocus
                />
              {:else}
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <span
                  class="editable-cell"
                  class:is-unknown={file.title === file.file_stem}
                  on:dblclick={() => startEdit(i, 'title', file.title)}
                  title="Double-click to edit"
                >
                  {file.title}
                </span>
              {/if}
            </td>

            <!-- Artist (editable) -->
            <td>
              {#if editing && editing.idx === i && editing.field === 'artist'}
                <!-- svelte-ignore a11y_autofocus -->
                <input
                  class="inline-input"
                  bind:value={editValue}
                  on:blur={commitEdit}
                  on:keydown={(e) => { if (e.key === 'Enter') commitEdit(); if (e.key === 'Escape') cancelEdit(); }}
                  autofocus
                />
              {:else}
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <span
                  class="editable-cell"
                  class:is-unknown={file.artist === 'Unknown'}
                  on:dblclick={() => startEdit(i, 'artist', file.artist)}
                  title="Double-click to edit"
                >
                  {file.artist}
                </span>
              {/if}
            </td>

            <!-- Album (editable) -->
            <td>
              {#if editing && editing.idx === i && editing.field === 'album'}
                <!-- svelte-ignore a11y_autofocus -->
                <input
                  class="inline-input"
                  bind:value={editValue}
                  on:blur={commitEdit}
                  on:keydown={(e) => { if (e.key === 'Enter') commitEdit(); if (e.key === 'Escape') cancelEdit(); }}
                  autofocus
                />
              {:else}
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <span
                  class="editable-cell"
                  class:is-unknown={file.album === 'Unknown'}
                  on:dblclick={() => startEdit(i, 'album', file.album)}
                  title="Double-click to edit"
                >
                  {file.album}
                </span>
              {/if}
            </td>

            <td class="mono">{file.bitrate > 0 ? file.bitrate + ' BIT' : '—'}</td>

            <!-- New name preview (only for non-duplicates) -->
            <td class="mono preview-cell">
              {#if file.status !== 'duplicate'}
                {targetName(file)}
              {:else}
                <span class="muted">→ /duplicates/</span>
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>

<style>
  .review-root { width: 100%; }

  .badges {
    display: flex;
    gap: 10px;
    margin-bottom: 14px;
    flex-wrap: wrap;
  }
  .badge {
    padding: 3px 10px;
    border-radius: 99px;
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 0.03em;
  }
  .badge-green  { background: rgba(74, 222, 128, 0.15); color: #4ade80; border: 1px solid rgba(74,222,128,0.3); }
  .badge-yellow { background: rgba(250, 204, 21, 0.15); color: #facc15; border: 1px solid rgba(250,204,21,0.3); }
  .badge-red    { background: rgba(248, 113, 113, 0.15); color: #f87171; border: 1px solid rgba(248,113,113,0.3); }

  .table-wrap {
    overflow-x: auto;
    border-radius: 10px;
    border: 1px solid var(--color-border);
  }
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.82rem;
  }
  thead th {
    background: var(--color-thead);
    color: var(--color-muted);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    font-size: 0.7rem;
    padding: 10px 12px;
    text-align: left;
    border-bottom: 1px solid var(--color-border);
  }
  tbody tr {
    border-bottom: 1px solid var(--color-border);
    transition: background 0.15s;
  }
  tbody tr:last-child { border-bottom: none; }

  .row-green  { background: rgba(74, 222, 128, 0.04); }
  .row-yellow { background: rgba(250, 204, 21, 0.06); }
  .row-red    { background: rgba(248, 113, 113, 0.06); }

  tbody tr:hover { filter: brightness(1.08); }

  td { padding: 9px 12px; vertical-align: middle; }

  .status-pill {
    display: inline-block;
    padding: 2px 8px;
    border-radius: 99px;
    font-size: 0.7rem;
    font-weight: 600;
    white-space: nowrap;
  }
  .pill-green  { background: rgba(74,222,128,0.2); color: #4ade80; }
  .pill-yellow { background: rgba(250,204,21,0.2); color: #facc15; }
  .pill-red    { background: rgba(248,113,113,0.2); color: #f87171; }

  .editable-cell {
    cursor: pointer;
    border-radius: 4px;
    padding: 1px 3px;
  }
  .editable-cell:hover {
    background: var(--color-hover);
    outline: 1px dashed var(--color-border);
  }
  .is-unknown { color: #facc15; font-style: italic; }

  .inline-input {
    background: var(--color-input-bg);
    border: 1px solid var(--color-accent);
    color: var(--color-text);
    border-radius: 4px;
    padding: 2px 6px;
    font-size: 0.82rem;
    width: 100%;
    outline: none;
  }

  .mono { font-family: 'JetBrains Mono', monospace; font-size: 0.75rem; }
  .preview-cell { color: var(--color-muted); max-width: 280px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .muted { color: var(--color-muted); }
</style>