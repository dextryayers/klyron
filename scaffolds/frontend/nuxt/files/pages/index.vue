<script setup lang="ts">
const count = ref(0)
</script>

<template>
  <div class="flex min-h-[80vh] flex-col items-center justify-center">
    <h1 class="text-4xl font-bold text-gray-900 sm:text-6xl">{{ name }}</h1>
    <p class="mt-4 text-lg text-gray-600">{{ description }}</p>
    <div class="mt-6">
      <button
        class="rounded-md bg-indigo-600 px-4 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500"
        @click="count++"
      >
        count is {{ count }}
      </button>
    </div>
  </div>
</template>
