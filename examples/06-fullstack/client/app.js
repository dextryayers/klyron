const taskList = document.getElementById("task-list");
const taskForm = document.getElementById("task-form");
const taskInput = document.getElementById("task-input");

async function loadTasks() {
  const res = await fetch("/api/tasks");
  const tasks = await res.json();
  taskList.innerHTML = tasks
    .map(
      (t) => `
      <li>
        <label>
          <input type="checkbox" ${t.done ? "checked" : ""} disabled />
          ${t.title}
        </label>
        <small>#${t.id}</small>
      </li>`
    )
    .join("");
}

taskForm.addEventListener("submit", async (e) => {
  e.preventDefault();
  const title = taskInput.value.trim();
  if (!title) return;

  await fetch("/api/tasks", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ title }),
  });

  taskInput.value = "";
  await loadTasks();
});

loadTasks();
