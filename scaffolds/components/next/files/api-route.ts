import { NextRequest, NextResponse } from 'next/server'

export async function GET(request: NextRequest) {
  return NextResponse.json({ message: 'Hello from API' })
}

export async function POST(request: NextRequest) {
  const body = await request.json()
  return NextResponse.json({ received: body, timestamp: new Date().toISOString() })
}
