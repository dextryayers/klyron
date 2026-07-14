const notesContainer = document.getElementById("notes-container");
const noteForm = document.getElementById("note-form");
const noteTitle = document.getElementById("note-title");
const noteContent = document.getElementById("note-content");

async function loadNotes() {
  const res = await fetch("/api/notes");
  const notes = await res.json();
  notesContainer.innerHTML = notes
    .map(
      (n) => `
      <article class="note-card">
        <header>
          <strong>${n.title}</strong>
          <button class="delete-btn" data-id="${n.id}" aria-label="Delete">✕</button>
        </header>
        <p class="content">${n.content}</p>
      </article>`
    )
    .join("");

  document.querySelectorAll(".delete-btn").forEach((btn) => {
    btn.addEventListener("click", async () => {
      const id = btn.dataset.id;
      await fetch(`/api/notes/${id}`, { method: "DELETE" });
      await loadNotes();
    });
  });
}

noteForm.addEventListener("submit", async (e) => {
  e.preventDefault();
  const title = noteTitle.value.trim();
  const content = noteContent.value.trim();
  if (!title || !content) return;

  await fetch("/api/notes", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ title, content }),
  });

  noteTitle.value = "";
  noteContent.value = "";
  await loadNotes();
});

loadNotes();
