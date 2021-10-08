<template>
  <h1 class="title">Rusty Vue Playground</h1>
  <header class="demo-header">
    <img class="crab" alt="Ferris with WA hat" src="../assets/wasm-ferris.png" />
    ❤️
    <img class="vue" alt="Vue logo" src="../assets/logo.png" />
  </header>
</template>

<style scoped>
.demo-header {
  max-width: 100%;
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  align-items: center;
  font-size: 60px;
  margin: 25px;
  gap: 20px;
}
.title {
  width: 100%;
  margin: 0;
}
.vue {
  margin-left: -5px;
}
.crab {
  margin-left: -120px;
}
.crab:hover {
  transform: rotate(30deg);
}
.demo-header > img {
  flex: 0 1 100px;
}
</style>
