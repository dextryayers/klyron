<script setup lang="ts">
import { ref } from 'vue'
import axios from 'axios'

const count = ref(0)

async function checkHealth() {
  const res = await axios.get('/api/health')
  console.log(res.data)
}
</script>

<template>
  <div class="flex min-h-screen flex-col items-center justify-center bg-gray-100">
    <h1 class="text-4xl font-bold sm:text-6xl">{{ name }}</h1>
    <p class="mt-4 text-lg text-gray-600">{{ description }}</p>
    <div class="mt-6 space-x-4">
      <button class="rounded-md bg-indigo-600 px-4 py-2 text-white shadow-sm hover:bg-indigo-500" @click="count++">count is {{ count }}</button>
      <button class="rounded-md bg-gray-600 px-4 py-2 text-white shadow-sm hover:bg-gray-500" @click="checkHealth">Health</button>
    </div>
  </div>
</template>
