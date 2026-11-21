import type { RouteObject } from "react-router";

import { OverridesPage } from "./OverridesPage";

const routes: RouteObject[] = [{ path: "/leagues/:id/overrides", element: <OverridesPage /> }];

export default routes;
