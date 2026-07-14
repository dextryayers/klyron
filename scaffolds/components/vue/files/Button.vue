<script setup>
defineProps({
  variant: { type: String, default: 'primary' },
  disabled: { type: Boolean, default: false },
})

defineEmits(['click'])
</script>

<template>
  <button
    :class="['btn', `btn--${variant}`, { 'btn--disabled': disabled }]"
    :disabled="disabled"
    @click="$emit('click', $event)"
  >
    <slot />
  </button>
</template>

<style scoped>
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0.6em 1.2em;
  border-radius: 8px;
  border: 1px solid transparent;
  font-size: 1em;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.25s;
}
.btn--disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
.btn--primary {
  background-color: #646cff;
  color: #fff;
}
.btn--secondary {
  background-color: #213547;
  color: #fff;
}
.btn--outline {
  background: transparent;
  border-color: #646cff;
  color: #646cff;
}
.btn--ghost {
  background: transparent;
  color: #213547;
}
.btn--danger {
  background-color: #dc2626;
  color: #fff;
}
.btn--primary:hover { background-color: #535bf2; }
.btn--secondary:hover { background-color: #1a2a38; }
.btn--outline:hover { background-color: #f0f0ff; }
.btn--ghost:hover { background-color: #f0f0f0; }
.btn--danger:hover { background-color: #b91c1c; }
</style>
