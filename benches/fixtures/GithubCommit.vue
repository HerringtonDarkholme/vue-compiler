<div id="demo">
  <h1>Latest Vue.js Commits</h1>
  <template v-for="branch in branches">
    <input
           type="radio"
           :id="branch"
           :value="branch"
           name="branch"
           v-model="currentBranch"
           />
    <label :for="branch">{{ branch }}</label>
  </template>
  <p>vuejs/vue@{{ currentBranch }}</p>
  <ul>
    <li v-for="record in commits">
      <a :href="record.html_url" target="_blank" class="commit"
         >{{ record.sha.slice(0, 7) }}</a
        >
      - <span class="message">{{ truncate(record.commit.message) }}</span
      ><br />
      by
      <span class="author"
            ><a :href="record.author.html_url" target="_blank"
                >{{ record.commit.author.name }}</a
        ></span
        >
      at
      <span class="date">{{ formatDate(record.commit.author.date) }}</span>
    </li>
  </ul>
</div>

