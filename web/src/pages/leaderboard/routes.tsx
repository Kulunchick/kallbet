import type { RouteObject } from "react-router";

import { LeaderboardPage } from "./LeaderboardPage";

const routes: RouteObject[] = [{ path: "/leagues/:id/leaderboard", element: <LeaderboardPage /> }];

export default routes;
