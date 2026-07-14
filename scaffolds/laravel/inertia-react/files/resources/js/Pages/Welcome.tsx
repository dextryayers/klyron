import { useState } from 'react'
import { Head } from '@inertiajs/react'

export default function Welcome() {
  const [count, setCount] = useState(0)

  return (
    <>
      <Head title="Welcome" />
      <div className="flex min-h-screen flex-col items-center justify-center bg-gray-100">
        <h1 className="text-4xl font-bold sm:text-6xl">{{ name }}</h1>
        <p className="mt-4 text-lg text-gray-600">{{ description }}</p>
        <div className="mt-6">
          <button
            className="rounded-md bg-indigo-600 px-4 py-2 text-white shadow-sm hover:bg-indigo-500"
            onClick={() => setCount((c) => c + 1)}
          >
            count is {count}
          </button>
        </div>
      </div>
    </>
  )
}
