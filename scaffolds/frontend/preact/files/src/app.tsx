import { useState } from 'preact/hooks'

export function App() {
  const [count, setCount] = useState(0)

  return (
    <div class="home">
      <h1>{{ name }}</h1>
      <p>{{ description }}</p>
      <div class="card">
        <button onClick={() => setCount((c) => c + 1)}>
          count is {count}
        </button>
      </div>
    </div>
  )
}
