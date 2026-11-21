import type { RouteObject } from "react-router";

import { JoinPage } from "./JoinPage";
import { LeaguePage } from "./LeaguePage";
import { MyLeaguesPage } from "./MyLeaguesPage";
import { NewLeaguePage } from "./NewLeaguePage";

const routes: RouteObject[] = [
  { path: "/leagues", element: <MyLeaguesPage /> },
  { path: "/leagues/new", element: <NewLeaguePage /> },
  { path: "/leagues/:id", element: <LeaguePage /> },
  { path: "/join/:token", element: <JoinPage /> },
];

export default routes;
