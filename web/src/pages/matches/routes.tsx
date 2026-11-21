import type { RouteObject } from "react-router";

import { MatchesPage } from "./MatchesPage";

const routes: RouteObject[] = [{ path: "/leagues/:id/matches", element: <MatchesPage /> }];

export default routes;
