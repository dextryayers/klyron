import { createSignal } from 'solid-js'

function App() {
  const [count, setCount] = createSignal(0)

  return (
    <div class="home">
      <h1>{{ name }}</h1>
      <p>{{ description }}</p>
      <div class="card">
        <button onclick={() => setCount((c) => c + 1)}>
          count is {count()}
        </button>
      </div>
    </div>
  )
}

export default App
