document.addEventListener("DOMContentLoaded", () => {
  const path = window.location.pathname.split("/");
  const owner = path[2];
  const name = path[3];

  document.getElementById("repo-name").textContent = `${owner}/${name}`;

  fetch(`/api/repositories/${owner}/${name}`)
    .then((response) => response.json())
    .then((metadata) => {
      console.log(metadata);
      const metadataDiv = document.getElementById("repo-metadata");
      metadataDiv.innerHTML = `
        <p>ID: ${metadata.id}</p>
        <p>Commit Count: ${metadata.commit_count}</p>
        <p>Latest Commit Date: ${metadata.latest_commit_date || "N/A"}</p>
        <p>Latest Commit Author: ${metadata.latest_commit_author || "N/A"}</p>
        <p>Indexed At: ${metadata.indexed_at || "Not indexed yet"}</p>
        <p>GitHub URL: <a href="${metadata.github_url}" target="_blank">${metadata.github_url}</a></p>
        `;
    });

  fetch(`/api/repositories/${owner}/${name}/ga`)
    .then((response) => response.json())
    .then((ga) => {
      console.log(ga);
      const gaDiv = document.getElementById("repo-ga");

      // Assuming ga is an array of objects with date, mau, retained, new, resurrected, and churned
      const latestGa = ga[ga.length - 1]; // Get the most recent data point

      gaDiv.innerHTML = `
          <h2>Growth Accounting (Latest)</h2>
          <p>Date: ${latestGa.date}</p>
          <p>MAU: ${latestGa.mau}</p>
          <p>Retained: ${latestGa.retained}</p>
          <p>New: ${latestGa.new}</p>
          <p>Resurrected: ${latestGa.resurrected}</p>
          <p>Churned: ${latestGa.churned}</p>
      `;
    });
});
