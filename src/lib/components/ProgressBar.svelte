<script lang="ts">
  export let value: number = 0; // 0–100
  export let label: string = '';
</script>

<div class="progress-wrap">
  {#if label}
    <div class="progress-label">{label}</div>
  {/if}
  <div class="progress-track">
    <div class="progress-bar" style="width: {value}%"></div>
  </div>
  <div class="progress-pct">{value.toFixed(0)}%</div>
</div>

<style>
  .progress-wrap {
    display: flex;
    flex-direction: column;
    gap: 4px;
    width: 100%;
  }
  .progress-label {
    font-size: 0.78rem;
    color: var(--color-muted);
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }
  .progress-track {
    width: 100%;
    height: 6px;
    background: var(--color-track);
    border-radius: 99px;
    overflow: hidden;
  }
  .progress-bar {
    height: 100%;
    background: var(--color-accent);
    border-radius: 99px;
    transition: width 0.25s ease;
  }
  .progress-pct {
    font-size: 0.72rem;
    color: var(--color-muted);
    align-self: flex-end;
  }
</style>