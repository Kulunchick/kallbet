import type { RouteObject } from "react-router";

import { PickemPage } from "./PickemPage";

const routes: RouteObject[] = [{ path: "/leagues/:id/pickem", element: <PickemPage /> }];

export default routes;
