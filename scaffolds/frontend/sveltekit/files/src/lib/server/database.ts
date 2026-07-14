export async function getHealth() {
  return { status: 'ok', service: '{{ name }}' }
}
