import { component$, useSignal } from '@builder.io/qwik'
import type { DocumentHead } from '@builder.io/qwik-city'

export default component$(() => {
  const count = useSignal(0)

  return (
    <div class="home">
      <h1>{{ name }}</h1>
      <p>{{ description }}</p>
      <div class="card">
        <button onClick$={() => count.value++}>
          count is {count.value}
        </button>
      </div>
    </div>
  )
})

export const head: DocumentHead = {
  title: '{{ name }}',
  meta: [{ name: 'description', content: '{{ description }}' }],
}
