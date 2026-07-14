import Link from 'next/link'

export default function Home() {
  return (
    <div className="flex min-h-[80vh] flex-col items-center justify-center">
      <h1 className="text-4xl font-bold tracking-tight text-gray-900 sm:text-6xl">
        {{ name }}
      </h1>
      <p className="mt-6 text-lg leading-8 text-gray-600">
        {{ description }}
      </p>
      <div className="mt-10 flex items-center gap-x-6">
        <Link
          href="/api/hello"
          className="rounded-md bg-primary-600 px-3.5 py-2.5 text-sm font-semibold text-white shadow-sm hover:bg-primary-500"
        >
          Get started
        </Link>
      </div>
    </div>
  )
}
