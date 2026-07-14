import { Title } from '@solidjs/meta'
import { createSignal } from 'solid-js'

export default function Home() {
  const [count, setCount] = createSignal(0)

  return (
    <main>
      <Title>{{ name }}</Title>
      <div class="home">
        <h1>{{ name }}</h1>
        <p>{{ description }}</p>
        <div class="card">
          <button onclick={() => setCount((c) => c + 1)}>
            count is {count()}
          </button>
        </div>
      </div>
    </main>
  )
}
