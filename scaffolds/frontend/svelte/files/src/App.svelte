<script lang="ts">
  let count = $state(0)
</script>

<main>
  <div class="home">
    <h1>{{ name }}</h1>
    <p>{{ description }}</p>
    <div class="card">
      <button onclick={() => count++}>
        count is {count}
      </button>
    </div>
  </div>
</main>

<style>
  .home {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 80vh;
    text-align: center;
  }

  .card {
    padding: 2em;
  }

  button {
    border-radius: 8px;
    border: 1px solid #ccc;
    padding: 0.6em 1.2em;
    font-size: 1em;
    font-weight: 500;
    cursor: pointer;
    transition: border-color 0.25s;
  }

  button:hover {
    border-color: #646cff;
  }
</style>
