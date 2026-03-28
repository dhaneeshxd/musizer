<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import ReviewTable from '$lib/components/ReviewTable.svelte';
  import ProgressBar from '$lib/components/ProgressBar.svelte';
  import HistoryView from '$lib/components/HistoryView.svelte';
  import CopyRecent from '$lib/components/CopyRecent.svelte';
  import type { AudioFile, RenameRecord, ProcessResult, AppPhase } from '$lib/types';

  let phase: AppPhase = 'idle';
  let sourceDir = '';
  let files: AudioFile[] = [];
  let historyRecords: RenameRecord[] = [];
  let currentSessionId = '';
  let progressValue = 0;
  let progressLabel = '';
  let errors: string[] = [];
  let activeTab: 'review' | 'history' | 'copy' = 'review';
  let processErrors: string[] = [];
  let showDone = false;

  function showDoneBanner() {
  showDone = true;

  setTimeout(() => {
    showDone = false;
  }, 2000); 
}

  async function pickFolder() {
    const selected = await open({ directory: true, multiple: false });
    if (!selected || typeof selected !== 'string') return;
    sourceDir = selected;
    await runScan();
  }

  async function runScan() {
    phase = 'scanning';
    progressValue = 10;
    progressLabel = 'Scanning folder…';
    errors = [];
    files = [];

    try {
      // Animate progress while scanning
      const ticker = setInterval(() => {
        if (progressValue < 85) progressValue += 5;
      }, 200);

      files = await invoke<AudioFile[]>('cmd_scan_folder', { folder: sourceDir });
      clearInterval(ticker);
      progressValue = 100;
      progressLabel = `Found ${files.length} audio files`;

      setTimeout(() => {
        phase = 'review';
        activeTab = 'review';
      }, 400);
    } catch (e: any) {
      errors = [String(e)];
      phase = 'idle';
    }
  }

  async function processFiles() {
    phase = 'processing';
    progressValue = 10;
    progressLabel = 'Renaming and moving files…';
    processErrors = [];

    try {
      const ticker = setInterval(() => {
        if (progressValue < 85) progressValue += 8;
      }, 300);

      const result = await invoke<ProcessResult>('cmd_process_files', {
        files,
        sourceDir
      });

      clearInterval(ticker);
      progressValue = 100;
      progressLabel = `Done — ${result.records.length} files processed`;
      currentSessionId = result.session_id;
      processErrors = result.errors;
      historyRecords = await invoke<RenameRecord[]>('cmd_get_recent_history', { limit: 200 });

      setTimeout(() => {
        phase = 'done';
        activeTab = 'history';
      }, 500);
      showDoneBanner();
    } catch (e: any) {
      errors = [String(e)];
      phase = 'review';
    }
  }


  async function loadHistory() {
    historyRecords = await invoke<RenameRecord[]>('cmd_get_recent_history', { limit: 200 }).catch(() => []);
    const sid = await invoke<string | null>('cmd_get_latest_session').catch(() => null);
    if (sid) currentSessionId = sid;
  }

  loadHistory();

  function reset() {
    phase = 'idle';
    files = [];
    sourceDir = '';
    progressValue = 0;
    progressLabel = '';
    errors = [];
    processErrors = [];
  }
</script>

<div class="app">
  <!-- Sidebar -->
  <aside class="sidebar">
    <div class="logo">
      <svg width="28" height="28" viewBox="0 0 28 28" fill="none">
        <circle cx="14" cy="14" r="13" stroke="var(--color-accent)" stroke-width="1.5"/>
        <circle cx="14" cy="14" r="5" fill="var(--color-accent)"/>
        <line x1="14" y1="1" x2="14" y2="6" stroke="var(--color-accent)" stroke-width="1.5"/>
        <line x1="14" y1="22" x2="14" y2="27" stroke="var(--color-accent)" stroke-width="1.5"/>
        <line x1="1" y1="14" x2="6" y2="14" stroke="var(--color-accent)" stroke-width="1.5"/>
        <line x1="22" y1="14" x2="27" y2="14" stroke="var(--color-accent)" stroke-width="1.5"/>
      </svg>
      <span class="logo-text">Musizer</span>
    </div>

    <nav class="nav">
      <button
        class="nav-item"
        class:active={activeTab === 'review'}
        on:click={() => { if (phase !== 'idle' && phase !== 'scanning') activeTab = 'review'; }}
      >
        <span class="nav-icon">◈</span> Review
      </button>
      <button
        class="nav-item"
        class:active={activeTab === 'history'}
        on:click={() => activeTab = 'history'}
      >
        <span class="nav-icon">⏱</span> History
      </button>
      <button
        class="nav-item"
        class:active={activeTab === 'copy'}
        on:click={() => activeTab = 'copy'}
      >
        <span class="nav-icon">⊕</span> Copy Recent
      </button>
    </nav>

    <div class="sidebar-footer">
      <div class="dir-display" title={sourceDir}>
        {sourceDir ? '📁 ' + sourceDir.split(/[/\\]/).pop() : 'No folder selected'}
      </div>
    </div>
  </aside>

  <!-- Main content -->
  <main class="main">
    <!-- Top bar -->
    <header class="topbar">
      <div class="topbar-left">
        <h1 class="page-title">
          {#if activeTab === 'review'}File Review
          {:else if activeTab === 'history'}Rename History
          {:else}Copy to Device
          {/if}
        </h1>
        {#if sourceDir && (phase === 'review' || phase === 'done')}
          <span class="dir-badge">{sourceDir}</span>
        {/if}
      </div>
      <div class="topbar-right">
        {#if phase === 'idle' || phase === 'done'}
          <button class="btn btn-primary" on:click={pickFolder}>
            {phase === 'done' ? '↺ Scan New Folder' : '+ Select Folder'}
          </button>
        {/if}
        {#if phase === 'review' && files.length > 0}
          <button class="btn btn-secondary" on:click={reset}>Cancel</button>
          <button class="btn btn-accent" on:click={processFiles}>
            ▶ Run Rename & Move
          </button>
        {/if}
      </div>
    </header>

    <!-- Progress (scanning or processing) -->
    {#if phase === 'scanning' || phase === 'processing'}
      <div class="progress-section">
        <ProgressBar value={progressValue} label={progressLabel} />
      </div>
    {/if}

    <!-- Done notification -->
    {#if phase === 'done' && progressLabel}
    {#if showDone}
      <div class="done-banner">
            ✓ {progressLabel}        
        {#if processErrors.length}
          — <span class="warn">{processErrors.length} error(s)</span>
        {/if}
      </div>
       {/if}
    {/if}

    <!-- Error display -->
    {#if errors.length > 0}
      <div class="error-box">
        {#each errors as e}<div>⚠ {e}</div>{/each}
      </div>
    {/if}

    <!-- Tab content -->
    <div class="content">
      {#if activeTab === 'review'}
        {#if phase === 'idle'}
          <div class="empty-state">
            <div class="empty-icon">♫</div>
            <div class="empty-title">Select a music folder to get started</div>
            <div class="empty-sub">Musizer will scan, deduplicate, and rename your audio files.</div>
            <button class="btn btn-primary btn-lg" on:click={pickFolder}>Select Folder</button>
          </div>
        {:else if phase === 'scanning'}
          <div class="scanning-state">Scanning…</div>
        {:else if files.length === 0}
          <div class="empty-state">
            <div class="empty-title">No audio files found in that folder.</div>
          </div>
        {:else}
          <ReviewTable {files} />
        {/if}
      {:else if activeTab === 'history'}
        <HistoryView />
      {:else if activeTab === 'copy'}
        <CopyRecent sessionId={currentSessionId} />
      {/if}
    </div>
  </main>
</div>

<style>
  :global(*) { box-sizing: border-box; margin: 0; padding: 0; }

  :global(:root) {
    --color-bg:       #0e0e12;
    --color-surface:  #14141a;
    --color-surface2: #1c1c24;
    --color-border:   #2a2a36;
    --color-thead:    #181820;
    --color-text:     #e8e8f0;
    --color-muted:    #6b6b80;
    --color-accent:   #a78bfa;
    --color-hover:    rgba(167,139,250,0.08);
    --color-track:    #2a2a36;
    --color-input-bg: #1c1c24;
    font-family: 'Syne', 'Inter', system-ui, sans-serif;
    font-size: 14px;
    color: var(--color-text);
    background: var(--color-bg);
  }

  :global(body) {
    background: var(--color-bg);
    color: var(--color-text);
    overflow: hidden;
  }

  .app {
    display: flex;
    height: 100vh;
    overflow: hidden;
  }

  /* ── Sidebar ── */
  .sidebar {
    width: 200px;
    min-width: 200px;
    background: var(--color-surface);
    border-right: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    padding: 20px 12px;
    gap: 28px;
  }
  .logo {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .logo-text {
    font-size: 1.15rem;
    font-weight: 800;
    letter-spacing: 0.04em;
    color: var(--color-text);
  }
  .nav {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .nav-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 10px;
    border-radius: 7px;
    border: none;
    background: transparent;
    color: var(--color-muted);
    font-size: 0.85rem;
    font-weight: 500;
    cursor: pointer;
    text-align: left;
    transition: background 0.15s, color 0.15s;
  }
  .nav-item:hover { background: var(--color-hover); color: var(--color-text); }
  .nav-item.active { background: rgba(167,139,250,0.15); color: var(--color-accent); font-weight: 700; }
  .nav-icon { font-size: 0.9rem; }

  .sidebar-footer {
    margin-top: auto;
  }
  .dir-display {
    font-size: 0.72rem;
    color: var(--color-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    padding: 6px 8px;
    background: var(--color-surface2);
    border-radius: 6px;
    border: 1px solid var(--color-border);
  }

  /* ── Main ── */
  .main {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 24px;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-surface);
    gap: 12px;
  }
  .topbar-left { display: flex; align-items: center; gap: 12px; }
  .topbar-right { display: flex; align-items: center; gap: 8px; }

  .page-title { font-size: 1.05rem; font-weight: 800; letter-spacing: 0.01em; }
  .dir-badge {
    font-size: 0.72rem;
    color: var(--color-muted);
    background: var(--color-surface2);
    border: 1px solid var(--color-border);
    border-radius: 4px;
    padding: 2px 8px;
    max-width: 300px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .btn {
    padding: 7px 16px;
    border-radius: 6px;
    font-size: 0.82rem;
    font-weight: 600;
    cursor: pointer;
    border: none;
    transition: opacity 0.15s, transform 0.1s;
    white-space: nowrap;
  }
  .btn:hover { opacity: 0.88; }
  .btn:active { transform: scale(0.97); }
  .btn-primary  { background: var(--color-accent); color: #0e0e12; }
  .btn-secondary { background: var(--color-surface2); color: var(--color-text); border: 1px solid var(--color-border); }
  .btn-accent   { background: #4ade80; color: #0e0e12; }
  .btn-lg { padding: 12px 28px; font-size: 0.92rem; }

  .progress-section {
    padding: 16px 24px 12px;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-surface);
  }

  .done-banner {
    margin: 14px 24px 0;
    padding: 10px 14px;
    background: rgba(74,222,128,0.1);
    border: 1px solid rgba(74,222,128,0.25);
    border-radius: 8px;
    font-size: 0.82rem;
    color: #4ade80;
  }
  .warn { color: #f87171; }

  .error-box {
    margin: 14px 24px 0;
    padding: 10px 14px;
    background: rgba(248,113,113,0.1);
    border: 1px solid rgba(248,113,113,0.25);
    border-radius: 8px;
    font-size: 0.8rem;
    color: #f87171;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: 20px 24px;
  }

  /* ── Empty / scanning states ── */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 14px;
    height: 100%;
    min-height: 300px;
    text-align: center;
  }
  .empty-icon { font-size: 3rem; color: var(--color-accent); opacity: 0.5; }
  .empty-title { font-size: 1.05rem; font-weight: 700; color: var(--color-text); }
  .empty-sub { font-size: 0.85rem; color: var(--color-muted); max-width: 340px; }
  .scanning-state { color: var(--color-muted); padding: 40px; text-align: center; }
</style>