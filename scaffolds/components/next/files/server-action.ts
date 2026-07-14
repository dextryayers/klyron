'use server'

import { revalidatePath } from 'next/cache'

export async function createItem(formData: FormData) {
  const name = formData.get('name')
  const email = formData.get('email')

  // TODO: Save to database
  console.log({ name, email })

  revalidatePath('/')
  return { success: true }
}

export async function deleteItem(id: string) {
  // TODO: Delete from database
  console.log('Deleting:', id)

  revalidatePath('/')
  return { success: true }
}
