<script setup lang="ts">
useSeoMeta({
  title: '{{ name }}',
  description: '{{ description }}',
})
</script>

<template>
  <div>
    <NuxtLayout>
      <NuxtPage />
    </NuxtLayout>
  </div>
</template>
