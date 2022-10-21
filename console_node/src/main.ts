import { createApp } from "vue";
import type { App } from "vue";
import { createPinia } from "pinia";

import Application from "./App.vue";
import router from "./router";

import "./assets/main.css";

const app: App = createApp(Application);

app.use(createPinia());
app.use(router);

app.mount("#app");
