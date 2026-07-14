import { headers } from 'next/headers'

async function getHealth() {
  try {
    const res = await fetch('http://localhost:8000/api/health', { cache: 'no-store' })
    return await res.json()
  } catch {
    return { status: 'unreachable' }
  }
}

export default async function Home() {
  const health = await getHealth()

  return (
    <div className="flex min-h-screen flex-col items-center justify-center bg-gray-100">
      <h1 className="text-4xl font-bold sm:text-6xl">{{ name }}</h1>
      <p className="mt-4 text-lg text-gray-600">{{ description }}</p>
      <div className="mt-4 text-sm text-gray-500">
        API Status: {health.status}
      </div>
    </div>
  )
}
