export default function ContactPage() {
  return (
    <div className="min-h-screen p-8">
      <h1 className="text-3xl font-bold mb-6">Contact Us</h1>
      <form className="max-w-md space-y-4">
        <div>
          <label htmlFor="name" className="block text-sm font-medium">Name</label>
          <input id="name" type="text" className="mt-1 block w-full rounded-md border p-2" />
        </div>
        <div>
          <label htmlFor="email" className="block text-sm font-medium">Email</label>
          <input id="email" type="email" className="mt-1 block w-full rounded-md border p-2" />
        </div>
        <div>
          <label htmlFor="message" className="block text-sm font-medium">Message</label>
          <textarea id="message" rows={4} className="mt-1 block w-full rounded-md border p-2" />
        </div>
        <button type="submit" className="rounded-md bg-blue-600 px-4 py-2 text-white">Send</button>
      </form>
    </div>
  )
}
