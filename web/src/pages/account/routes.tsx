import type { RouteObject } from "react-router";

import { LoginPage } from "./LoginPage";
import { ProfilePage } from "./ProfilePage";

const routes: RouteObject[] = [
  { path: "/login", element: <LoginPage /> },
  { path: "/profile", element: <ProfilePage /> },
];

export default routes;
