document.getElementById("myForm").addEventListener("submit", function (e) {
  e.preventDefault();

  const name = document.getElementById("name").value;
  const email = document.getElementById("email").value;

  // Send data to the server via POST
  fetch("/", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ name, email }),
  })
    .then((response) => response.json())
    .then((data) => {
      document.getElementById("responseMessage").textContent = data.message;
    })
    .catch((error) => {
      console.error("Error:", error);
    });
});
