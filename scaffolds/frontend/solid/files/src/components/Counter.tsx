import { createSignal } from 'solid-js'

export default function Counter() {
  const [count, setCount] = createSignal(0)

  return (
    <button
      class="rounded-md bg-indigo-600 px-4 py-2 text-white shadow-sm hover:bg-indigo-500"
      onclick={() => setCount((c) => c + 1)}
    >
      count is {count()}
    </button>
  )
}
