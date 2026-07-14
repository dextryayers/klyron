<template>
  <div class="hello">
    <h1>{{ msg }}</h1>
    <p>Welcome to {{ name }}</p>
  </div>
</template>

<script setup lang="ts">
defineProps<{ msg: string }>()
</script>

<style scoped>
.hello {
  text-align: center;
  padding: 2rem;
}
h1 {
  font-size: 2rem;
  color: #42b883;
}
</style>
