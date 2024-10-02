document.addEventListener("DOMContentLoaded", () => {
  fetch("/api/repositories")
    .then((response) => response.json())
    .then((repositories) => {
      const repoList = document.getElementById("repo-list");
      repositories.forEach((repo) => {
        const li = document.createElement("li");
        const a = document.createElement("a");
        a.href = `/repository/${repo.owner}/${repo.name}`;
        a.textContent = `${repo.owner}/${repo.name}`;
        li.appendChild(a);
        repoList.appendChild(li);
      });
    });
});
