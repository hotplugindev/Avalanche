import { NavLink, Outlet } from "react-router-dom";
import { cn } from "@/lib/utils";
import { useRepository } from "@/queries";
import { Badge } from "@/components/ui";

const navItems = [
  { to: "/", label: "Dashboard" },
  { to: "/hosts", label: "Hosts" },
  { to: "/users", label: "Users" },
  { to: "/profiles", label: "Profiles" },
  { to: "/applications", label: "Applications" },
  { to: "/capabilities", label: "Capabilities" },
  { to: "/graph", label: "Graph" },
  { to: "/provenance", label: "Provenance" },
  { to: "/doctor", label: "Repo Doctor" },
];

export function Layout() {
  const { data: repo } = useRepository();

  return (
    <div className="flex min-h-screen bg-zinc-950 text-zinc-100">
      <aside className="w-56 shrink-0 border-r border-zinc-800 p-4">
        <div className="mb-6 flex items-center gap-2">
          <span className="text-lg font-bold tracking-tight">Avalanche</span>
          {repo?.schemaVersion != null && (
            <Badge className="border-zinc-700 text-zinc-400">v{repo.schemaVersion}</Badge>
          )}
        </div>
        <nav className="space-y-1">
          {navItems.map((item) => (
            <NavLink
              key={item.to}
              to={item.to}
              end={item.to === "/"}
              className={({ isActive }) =>
                cn(
                  "block rounded-md px-3 py-2 text-sm transition-colors",
                  isActive
                    ? "bg-zinc-800 text-zinc-100 font-medium"
                    : "text-zinc-400 hover:bg-zinc-900 hover:text-zinc-200"
                )
              }
            >
              {item.label}
            </NavLink>
          ))}
        </nav>
        {repo && (
          <div className="mt-6 border-t border-zinc-800 pt-4 text-xs text-zinc-500">
            <div className="truncate" title={repo.root}>{repo.root}</div>
            <div className="mt-1 flex gap-3">
              <span>{repo.optionCount} options</span>
              {repo.errorCount > 0 && (
                <span className="text-red-400">{repo.errorCount} errors</span>
              )}
              {repo.warningCount > 0 && (
                <span className="text-amber-400">{repo.warningCount} warnings</span>
              )}
            </div>
          </div>
        )}
      </aside>
      <main className="flex-1 overflow-auto p-6">
        <Outlet />
      </main>
    </div>
  );
}
