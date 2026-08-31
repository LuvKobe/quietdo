// 静待 QuietDo —— 前端逻辑
// 通过 window.__TAURI__.core.invoke 调用 Rust 命令完成持久化与窗口操作。

const { invoke } = window.__TAURI__.core;

// ===== 状态 =====
let todos = [];
let config = {
  opacity: 88,
  showTitleBar: true,
  autoStart: false,
  locked: false,
  alwaysOnTop: false,
  trayTipShown: false,
};

// ===== DOM 引用 =====
let widget, titlebar, todoInput, addBtn, todoList;
let settingsBtn, settingsPanel, lockBtn, closeBtn, minBtn;
let opacitySlider, opacityValue, titleBarToggle, autoStartToggle, alwaysOnTopToggle;
let trayTip, trayTipOk;

// ===== 持久化 =====
async function loadData() {
  try {
    const t = JSON.parse(await invoke("load_todos"));
    if (Array.isArray(t)) todos = t;
  } catch (e) {
    console.error("加载任务失败", e);
  }
  try {
    const c = JSON.parse(await invoke("load_config"));
    if (c && typeof c === "object") config = Object.assign(config, c);
  } catch (e) {
    console.error("加载配置失败", e);
  }
  // 同步真实的开机自启状态（以系统注册为准）
  try {
    config.autoStart = await invoke("get_autostart");
  } catch (e) {}
}

async function saveTodos() {
  try {
    await invoke("save_todos", { data: JSON.stringify(todos) });
  } catch (e) {
    console.error("保存任务失败", e);
  }
}

async function saveConfig() {
  try {
    await invoke("save_config", { data: JSON.stringify(config) });
  } catch (e) {
    console.error("保存配置失败", e);
  }
}

// ===== 渲染任务列表 =====
function render() {
  todoList.innerHTML = "";
  if (todos.length === 0) {
    const hint = document.createElement("div");
    hint.className = "empty-hint";
    hint.textContent = "暂无任务，在上方添加";
    todoList.appendChild(hint);
    return;
  }
  todos.forEach((todo) => {
    const li = document.createElement("li");
    li.className = "todo-item" + (todo.done ? " done" : "");

    const check = document.createElement("div");
    check.className = "check";
    check.innerHTML = todo.done ? "✓" : "";
    check.addEventListener("click", () => toggleTodo(todo.id));

    const text = document.createElement("span");
    text.className = "todo-text";
    text.textContent = todo.text;
    text.title = "双击编辑";
    // 双击进入编辑
    text.addEventListener("dblclick", () => startEdit(todo, li, text));

    const del = document.createElement("button");
    del.className = "del-btn";
    del.innerHTML = "🗑";
    del.title = "删除";
    del.addEventListener("click", () => deleteTodo(todo.id));

    li.appendChild(check);
    li.appendChild(text);
    li.appendChild(del);
    todoList.appendChild(li);
  });
}

// ===== 任务操作 =====
function addTodo() {
  const text = todoInput.value.trim();
  if (!text) return; // 空内容忽略
  todos.unshift({ id: String(Date.now()), text, done: false }); // 新增到顶部
  saveTodos();
  render();
  todoInput.value = "";
  todoInput.focus(); // 连续录入
}

function toggleTodo(id) {
  const t = todos.find((t) => t.id === id);
  if (t) {
    t.done = !t.done;
    saveTodos();
    render();
  }
}

function deleteTodo(id) {
  // 直接物理删除，不保留副本
  todos = todos.filter((t) => t.id !== id);
  saveTodos();
  render();
}

// 收进托盘：首次弹出提示（用户点"知道了"后隐藏并记住），之后直接隐藏
function hideToTray() {
  if (!config.trayTipShown) {
    config.trayTipShown = true;
    saveConfig();
    trayTip.classList.remove("hidden");
  } else {
    invoke("hide_window");
  }
}

// 双击编辑任务文字：把文字替换成输入框，回车/失焦保存，Esc 取消
function startEdit(todo, li, textEl) {
  const input = document.createElement("input");
  input.type = "text";
  input.className = "todo-edit-input";
  input.value = todo.text;
  input.maxLength = 200;

  li.replaceChild(input, textEl);
  input.focus();
  input.select();

  let finished = false;
  const commit = (save) => {
    if (finished) return;
    finished = true;
    if (save) {
      const val = input.value.trim();
      if (val) todo.text = val; // 非空才更新，空则保持原值
      saveTodos();
    }
    render();
  };

  input.addEventListener("keydown", (e) => {
    if (e.key === "Enter") commit(true);
    else if (e.key === "Escape") commit(false);
  });
  input.addEventListener("blur", () => commit(true));
}

// ===== 应用配置到界面 =====
function applyConfig() {
  widget.style.opacity = config.opacity / 100;
  opacitySlider.value = config.opacity;
  opacityValue.textContent = config.opacity + "%";

  titleBarToggle.checked = config.showTitleBar;
  titlebar.classList.toggle("hidden", !config.showTitleBar);

  autoStartToggle.checked = config.autoStart;
  alwaysOnTopToggle.checked = config.alwaysOnTop;
  // 按配置应用窗口置顶
  invoke("set_always_on_top", { enabled: config.alwaysOnTop }).catch(() => {});

  lockBtn.textContent = config.locked ? "🔒" : "🔓";
  lockBtn.classList.toggle("locked", config.locked);
  titlebar.classList.toggle("locked", config.locked);
  widget.classList.toggle("locked", config.locked);
}

// ===== 事件绑定 =====
function bindEvents() {
  addBtn.addEventListener("click", addTodo);
  todoInput.addEventListener("keydown", (e) => {
    if (e.key === "Enter") addTodo();
  });

  // 透明度：实时预览，松手保存
  opacitySlider.addEventListener("input", () => {
    config.opacity = parseInt(opacitySlider.value, 10);
    widget.style.opacity = config.opacity / 100;
    opacityValue.textContent = config.opacity + "%";
  });
  opacitySlider.addEventListener("change", saveConfig);

  // 显示/隐藏标题栏
  titleBarToggle.addEventListener("change", () => {
    config.showTitleBar = titleBarToggle.checked;
    titlebar.classList.toggle("hidden", !config.showTitleBar);
    saveConfig();
  });

  // 开机自启（写入系统 + 记录配置）
  autoStartToggle.addEventListener("change", async () => {
    config.autoStart = autoStartToggle.checked;
    try {
      await invoke("set_autostart", { enabled: config.autoStart });
    } catch (e) {
      console.error("设置开机自启失败", e);
    }
    saveConfig();
  });

  // 窗口置顶
  alwaysOnTopToggle.addEventListener("change", async () => {
    config.alwaysOnTop = alwaysOnTopToggle.checked;
    try {
      await invoke("set_always_on_top", { enabled: config.alwaysOnTop });
    } catch (e) {
      console.error("设置窗口置顶失败", e);
    }
    saveConfig();
  });

  // 设置面板开关
  settingsBtn.addEventListener("click", (e) => {
    e.stopPropagation();
    settingsPanel.classList.toggle("hidden");
  });
  document.addEventListener("click", (e) => {
    if (
      !settingsPanel.classList.contains("hidden") &&
      !settingsPanel.contains(e.target) &&
      e.target !== settingsBtn
    ) {
      settingsPanel.classList.add("hidden");
    }
  });

  // 锁定按钮
  lockBtn.addEventListener("click", () => {
    config.locked = !config.locked;
    saveConfig();
    applyConfig();
  });

  // 最小化按钮：收进托盘
  minBtn.addEventListener("click", () => hideToTray());

  // 关闭按钮：退出程序
  closeBtn.addEventListener("click", () => invoke("close_app"));

  // 首次提示的"知道了"：关闭提示并真正隐藏
  trayTipOk.addEventListener("click", () => {
    trayTip.classList.add("hidden");
    invoke("hide_window");
  });

  // 标题栏拖动（调用系统窗口拖动；锁定或点按钮时不触发）
  titlebar.addEventListener("mousedown", (e) => {
    if (config.locked) return;
    if (e.target.closest(".icon-btn")) return;
    invoke("start_drag");
  });

  // 窗口缩放：四边 + 四角手柄（锁定时禁用）
  document.querySelectorAll(".resizer").forEach((el) => {
    el.addEventListener("mousedown", (e) => {
      if (config.locked) return;
      e.preventDefault();
      invoke("start_resize", { direction: el.dataset.dir });
    });
  });
}

// ===== 初始化 =====
window.addEventListener("DOMContentLoaded", async () => {
  widget = document.getElementById("widget");
  titlebar = document.getElementById("titlebar");
  todoInput = document.getElementById("todoInput");
  addBtn = document.getElementById("addBtn");
  todoList = document.getElementById("todoList");
  settingsBtn = document.getElementById("settingsBtn");
  settingsPanel = document.getElementById("settingsPanel");
  lockBtn = document.getElementById("lockBtn");
  closeBtn = document.getElementById("closeBtn");
  minBtn = document.getElementById("minBtn");
  trayTip = document.getElementById("trayTip");
  trayTipOk = document.getElementById("trayTipOk");
  opacitySlider = document.getElementById("opacitySlider");
  opacityValue = document.getElementById("opacityValue");
  titleBarToggle = document.getElementById("titleBarToggle");
  autoStartToggle = document.getElementById("autoStartToggle");
  alwaysOnTopToggle = document.getElementById("alwaysOnTopToggle");

  await loadData();
  applyConfig();
  render();
  bindEvents();
  todoInput.focus();
});
