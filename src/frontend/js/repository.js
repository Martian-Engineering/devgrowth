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
    .then((gaData) => {
      console.log(gaData);
      createGrowthAccountingChart(gaData);
    });
});

function createGrowthAccountingChart(gaData) {
  const ctx = document.getElementById("gaChart").getContext("2d");

  const labels = gaData.map((d) => d.date);
  const retained = gaData.map((d) => d.retained);
  const newUsers = gaData.map((d) => d.new);
  const resurrected = gaData.map((d) => d.resurrected);
  const churned = gaData.map((d) => d.churned);

  new Chart(ctx, {
    type: "bar",
    data: {
      labels: labels,
      datasets: [
        {
          label: "Retained",
          data: retained,
          backgroundColor: "rgba(75, 192, 192, 0.6)",
        },
        {
          label: "New",
          data: newUsers,
          backgroundColor: "rgba(54, 162, 235, 0.6)",
        },
        {
          label: "Resurrected",
          data: resurrected,
          backgroundColor: "rgba(255, 206, 86, 0.6)",
        },
        {
          label: "Churned",
          data: churned,
          backgroundColor: "rgba(255, 99, 132, 0.6)",
        },
      ],
    },
    options: {
      responsive: true,
      scales: {
        x: {
          stacked: true,
        },
        y: {
          stacked: true,
          title: {
            display: true,
            text: "Number of Developers",
          },
        },
      },
      plugins: {
        tooltip: {
          callbacks: {
            label: function (context) {
              let label = context.dataset.label || "";
              if (label) {
                label += ": ";
              }
              if (context.parsed.y !== null) {
                label += context.parsed.y;
              }
              return label;
            },
          },
        },
        title: {
          display: true,
          text: "Growth Accounting Over Time",
        },
        legend: {
          position: "top",
        },
      },
    },
  });
}
