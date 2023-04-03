import { Route, Routes } from "react-router-dom";
import { useInitTamagotchi } from "@/app/hooks/use-tamagotchi";
import { useThrottleWasmState } from "@/app/hooks/use-read-wasm-state";
import { useItemsStore } from "@/app/hooks/use-ft-store";
import { lazy } from "react";

const routes = [
  { path: "/", Page: lazy(() => import("./home")) },
  { path: "/store", Page: lazy(() => import("./store")) },
  { path: "/battle", Page: lazy(() => import("./battle")) },
];

export const Routing = () => {
  useInitTamagotchi();
  useThrottleWasmState();
  useItemsStore();

  return (
    <Routes>
      {routes.map(({ path, Page }) => (
        <Route key={path} path={path} element={<Page />} />
      ))}
    </Routes>
  );
};
