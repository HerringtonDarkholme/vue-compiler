<template>
  <div id="conditional-rendering">
    <span v-if="seen">Now you see me</span>
  </div>
</template>
<script>
export default {
  props: {
    seen: Boolean,
  }
}
</script>
