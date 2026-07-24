<script>
  import { open } from '@tauri-apps/plugin-dialog'
  import { listDirectory } from '@sito/plugin-workspace'

  let folder = $state('')
  let entries = $state([])
  let error = $state('')

  async function chooseFolder() {
    const selected = await open({ directory: true, multiple: false })
    if (typeof selected !== 'string') return
    folder = selected
    error = ''

    try {
      entries = await listDirectory(selected)
    } catch (reason) {
      entries = []
      error = reason?.message ?? String(reason)
    }
  }
</script>

<main class="container">
  <h1>Sito workspace plugin</h1>
  <button onclick={chooseFolder}>Choose folder</button>
  {#if folder}<p>{folder}</p>{/if}
  {#if error}<p class="error">{error}</p>{/if}
  <ul>
    {#each entries as entry (entry.path)}
      <li>{entry.kind}: {entry.name}</li>
    {/each}
  </ul>
</main>

<style>
  .error {
    color: #b00020;
  }
</style>
