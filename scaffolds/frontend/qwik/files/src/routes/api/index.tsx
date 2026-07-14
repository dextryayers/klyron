import { routeLoader$ } from '@builder.io/qwik-city'

export const useApiInfo = routeLoader$(async () => {
  return {
    status: 'ok',
    service: '{{ name }}',
    version: '{{ version }}',
  }
})

export default function ApiPage() {
  const info = useApiInfo()
  return <pre>{JSON.stringify(info.value, null, 2)}</pre>
}
