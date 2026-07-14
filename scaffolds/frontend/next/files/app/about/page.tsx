import type { Metadata } from 'next'

export const metadata: Metadata = { title: 'About | {{ name }}' }

export default function About() {
  return (
    <div className="mx-auto max-w-7xl px-4 py-12">
      <h1 className="text-3xl font-bold">About {{ name }}</h1>
      <p className="mt-4 text-gray-600">{{ description }}</p>
    </div>
  )
}
