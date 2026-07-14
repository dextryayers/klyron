<script setup lang="ts">
const props = defineProps({
  error: Object,
})
</script>

<template>
  <div class="flex min-h-screen flex-col items-center justify-center">
    <h1 class="text-4xl font-bold text-gray-900">
      {{ error?.statusCode || 'Error' }}
    </h1>
    <p class="mt-2 text-gray-600">{{ error?.message || 'Something went wrong' }}</p>
    <NuxtLink to="/" class="mt-4 text-indigo-600 hover:underline">
      Go back home
    </NuxtLink>
  </div>
</template>
