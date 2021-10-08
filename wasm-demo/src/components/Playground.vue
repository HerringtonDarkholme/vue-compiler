<script setup lang="ts">
import { ref, watchEffect } from 'vue'
import init, {compile} from 'rusty-vue-compiler'
await init()

let input = ref('<p>Hello World from wasm!</p>')
let output = ref('')

watchEffect(async () => {
  output.value = compile(input.value)
})

</script>

<template>
    <div class="playground">
      <textarea v-model="input"/>
      <code v-html="output"/>
    </div>
</template>

<style scoped>
.playground {
  display: flex;
  flex-wrap: wrap;
  flex-grow: 1;
  text-align: left;
  font-family: monospace;
}
.playground > * {
  flex: 1 0 320px;
  min-width: 320px;
  max-width: 100%;
  overflow-x: auto;
  padding: 1em;
  margin: 0 1em 2em 1em;
}
textarea {
  resize: none;
}

label {
  margin: 0 0.5em;
  font-weight: bold;
}

code {
  white-space: pre-wrap;
  background-color: #eee;
  padding: 2px 4px;
  border-radius: 4px;
  color: #304455;
}
</style>
