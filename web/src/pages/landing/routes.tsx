import type { RouteObject } from "react-router";

import { LandingPage } from "./LandingPage";
import { NotFoundPage } from "./NotFoundPage";

const routes: RouteObject[] = [
  { path: "/", element: <LandingPage /> },
  { path: "*", element: <NotFoundPage /> },
];

export default routes;
