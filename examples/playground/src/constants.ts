export const DEFAULT_APP_HXO = `<template>
  <div class="p-6 bg-white rounded-xl shadow-sm border border-gray-100 max-w-md mx-auto">
    <h1 class="text-2xl font-bold text-gray-800 mb-4">{{ title }}</h1>
    
    <div class="flex items-center gap-4 mb-6">
      <!-- 在 template 中，ref 会被自动解包，无需 .value -->
      <div class="text-3xl font-mono bg-gray-50 px-4 py-2 rounded-lg border border-gray-200">
        {{ count }}
      </div>
      <div class="flex flex-col gap-2">
        <button @click="increment" class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors font-medium">
          Increment
        </button>
        <button @click="count = 0" class="text-sm text-gray-500 hover:text-gray-700 underline">
          Reset
        </button>
      </div>
    </div>

    <div class="p-3 bg-blue-50 text-blue-700 rounded-lg text-sm" v-if="count > 10">
      Wow! You've clicked more than 10 times!
    </div>
  </div>
</template>

<script>
// HXO 编译器支持 Reactive Transform
// 使用 ref 声明响应式变量，在 script 中可以直接使用，无需 .value
let count = ref(0);
const title = "A Better Vue";

function increment() {
  count++; // 编译器会自动处理响应式赋值
}

watchEffect(() => {
  if (count > 0) {
    console.log("Count changed:", count);
  }
});
</script>

<style scoped>
h1 { letter-spacing: -0.025em; }
</style>`;

export const EXAMPLES = [
    {
        name: "Counter",
        files: [
            {
                name: "App.hxo",
                content: `<template>
  <div class="flex items-center gap-4 p-4 border rounded-lg w-fit">
    <button @click="count--" class="w-10 h-10 flex items-center justify-center bg-gray-100 hover:bg-gray-200 rounded-full">-</button>
    <span class="text-xl font-bold min-w-[2rem] text-center">{{ count }}</span>
    <button @click="count++" class="w-10 h-10 flex items-center justify-center bg-gray-100 hover:bg-gray-200 rounded-full">+</button>
  </div>
</template>

<script>
let count = ref(0); // 像普通变量一样声明，底层是 signal
</script>`,
                language: "hxo",
            },
        ],
    },
    {
        name: "Todo List",
        files: [
            {
                name: "App.hxo",
                content: `<template>
  <div class="max-w-md mx-auto p-6 bg-white rounded-xl shadow-lg border">
    <h2 class="text-xl font-bold mb-4">Todo List</h2>
    
    <div class="flex gap-2 mb-4">
      <input 
        @keyup.enter="addTodo" 
        placeholder="Add a task..." 
        class="flex-1 px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
      />
    </div>

    <ul class="space-y-2">
      <li v-for="todo in todos" class="flex items-center gap-3 p-3 bg-gray-50 rounded-lg group">
        <input 
          type="checkbox" 
          v-model="todo.completed"
          class="w-5 h-5 rounded border-gray-300 text-blue-600"
        />
        <span :class="{ 'line-through text-gray-400': todo.completed, 'text-gray-700': !todo.completed }">
          {{ todo.text }}
        </span>
      </li>
    </ul>
    
    <div class="mt-4 text-sm text-gray-500">
      {{ activeCount }} items left
    </div>
  </div>
</template>

<script>
// 直接操作数组，HXO 编译器会自动处理响应式追踪
let todos = ref([
  { text: "Learn HXO", completed: true },
  { text: "Build something cool", completed: false }
]);

const activeCount = computed(() => todos.filter(t => !t.completed).length);

function addTodo(e) {
  const text = e.target.value.trim();
  if (!text) return;
  todos.push({ text, completed: false }); // 就像原生 JS 一样简单
  e.target.value = "";
}
</script>`,
                language: "hxo",
            },
        ],
    },
    {
        name: "Tailwind Demo",
        files: [
            {
                name: "App.hxo",
                content: DEFAULT_APP_HXO,
                language: "hxo",
            },
        ],
    },
];
