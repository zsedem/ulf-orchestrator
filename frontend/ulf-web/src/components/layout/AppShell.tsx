/**
 * AppShell Component
 *
 * Main application layout with fixed sidebar and scrollable content area.
 * Uses React Router's Outlet for nested route rendering.
 * Provides the structural shell for the entire application.
 */

import { Outlet } from "react-router-dom";
import { Sidebar } from "./Sidebar";

export function AppShell() {
  return (
    <div className="flex h-screen overflow-hidden bg-background">
      {/* Fixed sidebar */}
      <Sidebar />

      {/* Main content area - renders active route via Outlet */}
      <main className="flex-1 overflow-auto">
        <div className="bg-amber-500/10 text-amber-700 dark:text-amber-400 text-sm px-4 py-2 text-center border-b border-amber-500/20">
          The web dashboard is deprecated. Use <code className="font-mono bg-amber-500/20 px-1 rounded">ulf workspace</code> CLI commands instead.
        </div>
        <div className="p-6">
          <Outlet />
        </div>
      </main>
    </div>
  );
}
