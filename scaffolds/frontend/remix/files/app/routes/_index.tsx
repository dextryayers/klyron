import { useState } from 'react'
import type { MetaFunction } from '@remix-run/node'

export const meta: MetaFunction = () => {
  return [
    { title: '{{ name }}' },
    { name: 'description', content: '{{ description }}' },
  ]
}

export default function Index() {
  const [count, setCount] = useState(0)

  return (
    <div className="flex min-h-[80vh] flex-col items-center justify-center">
      <h1 className="text-4xl font-bold text-gray-900 sm:text-6xl">{{ name }}</h1>
      <p className="mt-4 text-lg text-gray-600">{{ description }}</p>
      <div className="mt-6">
        <button
          className="rounded-md bg-indigo-600 px-4 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500"
          onClick={() => setCount((c) => c + 1)}
        >
          count is {count}
        </button>
      </div>
    </div>
  )
}
