import "@fontsource-variable/inter";
import "@fontsource-variable/unbounded";
import "./styles/theme.css";
import "./i18n";

import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { RouterProvider } from "react-router";

import { Toasts } from "./components/Toasts";
import { router } from "./app/router";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <Toasts />
    <RouterProvider router={router} />
  </StrictMode>,
);
