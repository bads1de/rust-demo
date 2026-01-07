import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import App from "./App.tsx";

console.log("Starting app...");

window.addEventListener("error", (event) => {
  document.body.innerHTML += `<div style="color: red; padding: 20px; font-size: 20px;">Global Error: ${event.message}</div>`;
  console.error("Global Error:", event.error);
});

window.addEventListener("unhandledrejection", (event) => {
  document.body.innerHTML += `<div style="color: red; padding: 20px; font-size: 20px;">Promise Rejection: ${event.reason}</div>`;
  console.error("Promise Rejection:", event.reason);
});

try {
  const rootElement = document.getElementById("root");
  if (!rootElement) throw new Error("Root element not found");

  const root = createRoot(rootElement);
  console.log("Root created, rendering...");

  root.render(
    <StrictMode>
      <App />
    </StrictMode>
  );
  console.log("Render called");
} catch (e) {
  document.body.innerHTML += `<div style="color: red; padding: 20px; font-size: 20px;">Startup Error: ${e}</div>`;
  console.error("Startup Error:", e);
}
