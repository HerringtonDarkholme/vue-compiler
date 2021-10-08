<script setup lang="ts">
// This starter template is using Vue 3 <script setup> SFCs
// Check out https://v3.vuejs.org/api/sfc-script-setup.html#sfc-script-setup
import Playground from './components/Playground.vue'
import Header from './components/Header.vue'
import Footer from './components/Footer.vue'
</script>

<template>
  <Header/>
  <main>
    <suspense>
      <Playground />
      <template #fallback>
        <h1>
          Loading WASM...
        </h1>
      </template>
    </suspense>
  </main>
  <Footer/>
</template>

<style scoped>
main {
  flex-grow: 1;
  display: flex;
  justify-content: center;
}
</style>

<style>
body, html {
  margin: 0;
  padding: 0;
}
#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: #2c3e50;
  max-width: 100%;
  display: flex;
  flex-direction: column;
  min-height: 100vh;
}
</style>
