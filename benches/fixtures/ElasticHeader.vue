<div class="draggable-header-view"
   @mousedown="startDrag" @touchstart="startDrag"
   @mousemove="onDrag" @touchmove="onDrag"
   @mouseup="stopDrag" @touchend="stopDrag" @mouseleave="stopDrag">
  <svg class="bg" width="320" height="560">
    <path :d="headerPath" fill="#3F51B5"></path>
  </svg>
  <div class="header">
    <slot name="header"></slot>
  </div>
  <div class="content" :style="contentPosition">
    <slot name="content"></slot>
  </div>
</div>

<div id="app" @touchmove.prevent>
  <draggable-header-view>
    <template v-slot:header>
      <h1>Elastic Draggable SVG Header</h1>
      <p>
        with <a href="http://vuejs.org">Vue.js</a> +
        <a href="http://dynamicsjs.com">dynamics.js</a>
      </p>
    </template>
    <template v-slot:content>
      <p>
        Note this is just an effect demo - there are of course many
        additional details if you want to use this in production, e.g.
        handling responsive sizes, reload threshold and content scrolling.
        Those are out of scope for this quick little hack. However, the idea
        is that you can hide them as internal details of a Vue.js component
        and expose a simple Web-Component-like interface.
      </p>
    </template>
  </draggable-header-view>
</div>
