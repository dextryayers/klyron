<script setup lang="ts">
defineProps<{
  name: string
}>()
</script>

<template>
  <div class="rounded-lg border border-gray-200 bg-white p-6 shadow-sm">
    <h3 class="text-lg font-semibold text-gray-900">Hello {{ name }}</h3>
    <p class="mt-2 text-sm text-gray-600">
      This is a Nuxt 3 component.
    </p>
  </div>
</template>
