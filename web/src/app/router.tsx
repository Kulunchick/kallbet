import { createBrowserRouter, type RouteObject } from "react-router";

import { Layout } from "./Layout";

// Every feature area owns `src/pages/<area>/routes.tsx`, which default-exports
// its RouteObject[]. They are collected here automatically, so adding a page
// never means editing this file.
const areas = import.meta.glob<RouteObject[]>("../pages/*/routes.tsx", {
  eager: true,
  import: "default",
});

export const router = createBrowserRouter([
  {
    element: <Layout />,
    children: Object.values(areas).flat(),
  },
]);
