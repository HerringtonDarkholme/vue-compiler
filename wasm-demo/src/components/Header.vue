<template>
  <header class="demo-header">
    <h1 class="title">Rusty Vue Playground</h1>
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
  font-size: min(60px, 9vmin);
  padding: 20px;
  gap: min(15px, 1vmin);
}
.title {
  font-size: 30px;
  width: 100%;
  margin: 0;
}
.vue {
  margin-left: -5px;
  width: min(200px, 30vmin);
}
.crab {
  width: min(317px, 47vmin);
}
@media only screen and (min-width: 640px) {
  .demo-header {
    gap: 20px;
  }
  .crab {
    margin-left: -120px;
  }
}

.crab:hover {
  animation: 1s shakeX;
}
/*https://github.com/animate-css/animate.css/blob/main/source/attention_seekers/shakeX.css*/
@keyframes shakeX {
  from,
  to {
    transform: translate3d(0, 0, 0);
  }

  10%,
  30%,
  50%,
  70%,
  90% {
    transform: translate3d(-10px, 0, 0);
  }

  20%,
  40%,
  60%,
  80% {
    transform: translate3d(10px, 0, 0);
  }
}
.demo-header > img {
  flex: 0 1 100px;
}
</style>
