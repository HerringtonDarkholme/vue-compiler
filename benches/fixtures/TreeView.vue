<li>
  <div
       :class="{bold: isFolder}"
       @click="toggle"
       @dblclick="makeFolder">
    {{ item.name }}
    <span v-if="isFolder">[{{ isOpen ? '-' : '+' }}]</span>
  </div>
  <ul v-show="isOpen" v-if="isFolder">
    <tree-item
       class="item"
       v-for="(child, index) in item.children"
       :key="index"
       :item="child"
       @make-folder="$emit('make-folder', $event)"
       @add-item="$emit('add-item', $event)"
       ></tree-item>
    <li class="add" @click="$emit('add-item', item)">+</li>
  </ul>
</li>
