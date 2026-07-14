<script setup>
defineProps({
  title: { type: String, default: '' },
  image: { type: String, default: '' },
  variant: { type: String, default: 'elevated' },
  footer: { type: Boolean, default: false },
})
</script>

<template>
  <div :class="['card', `card--${variant}`]">
    <img v-if="image" :src="image" alt="" class="card__image" />
    <div class="card__body">
      <h3 v-if="title" class="card__title">{{ title }}</h3>
      <div class="card__content">
        <slot />
      </div>
    </div>
    <div v-if="$slots.footer" class="card__footer">
      <slot name="footer" />
    </div>
  </div>
</template>

<style scoped>
.card {
  border-radius: 12px;
  overflow: hidden;
  background-color: #fff;
}
.card--elevated {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}
.card--outlined {
  border: 1px solid #e0e0e0;
}
.card--flat {
  background-color: #f9f9f9;
}
.card__image {
  width: 100%;
  height: 200px;
  object-fit: cover;
}
.card__body {
  padding: 1.5rem;
}
.card__title {
  margin: 0 0 0.75rem 0;
  font-size: 1.25rem;
  font-weight: 600;
}
.card__content {
  color: #555;
  line-height: 1.5;
}
.card__footer {
  padding: 1rem 1.5rem;
  border-top: 1px solid #e0e0e0;
  background-color: #f9f9f9;
  display: flex;
  gap: 0.5rem;
}
</style>
