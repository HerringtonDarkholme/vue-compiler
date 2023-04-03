<template>
  <div id="conditional-rendering">
    <span v-if="seen">Now you see me</span>
  </div>
</template>
<script>
export default {
  option: {
    seen: Boolean,
  }
}
</script>
