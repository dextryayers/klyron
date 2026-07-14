<script setup lang="ts">
</script>

<template>
  <div>
    <header class="border-b border-gray-200 bg-white">
      <div class="mx-auto max-w-7xl px-4 py-4">
        <NuxtLink to="/" class="text-xl font-bold">{{ name }}</NuxtLink>
      </div>
    </header>
    <main>
      <slot />
    </main>
  </div>
</template>
