import { Link } from "react-router-dom";
import { useRepository } from "@/queries";
import { Card, CardContent, CardHeader, CardTitle, LoadingBlock, ErrorBlock, Badge } from "@/components/ui";

function Stat({ label, value, to }: { label: string; value: number | string; to?: string }) {
  const inner = (
    <Card className="hover:border-zinc-700 transition-colors">
      <CardHeader className="pb-2">
        <CardTitle className="text-sm font-medium text-zinc-400">{label}</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="text-2xl font-bold">{value}</div>
      </CardContent>
    </Card>
  );
  return to ? <Link to={to}>{inner}</Link> : inner;
}

export function Dashboard() {
  const { data: repo, isLoading, error } = useRepository();

  if (isLoading) return <LoadingBlock />;
  if (error) return <ErrorBlock message={String(error)} />;
  if (!repo) return null;

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Dashboard</h1>
        <p className="text-sm text-zinc-400 mt-1">{repo.root}</p>
      </div>

      <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4">
        <Stat label="Hosts" value={repo.hostCount} to="/hosts" />
        <Stat label="Users" value={repo.userCount} to="/users" />
        <Stat label="Profiles" value={repo.profileCount} to="/profiles" />
        <Stat label="Applications" value={repo.applicationCount} to="/applications" />
        <Stat label="Capabilities" value={repo.capabilityCount} to="/capabilities" />
        <Stat label="Options" value={repo.optionCount} />
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Health</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-zinc-400">Active requests</span>
              <span>{repo.activeRequestCount}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-zinc-400">Errors</span>
              <span className={repo.errorCount > 0 ? "text-red-400" : "text-emerald-400"}>
                {repo.errorCount}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-zinc-400">Warnings</span>
              <span className={repo.warningCount > 0 ? "text-amber-400" : "text-emerald-400"}>
                {repo.warningCount}
              </span>
            </div>
            {(repo.errorCount > 0 || repo.warningCount > 0) && (
              <Link to="/doctor" className="inline-block mt-2 text-zinc-300 hover:underline">
                View in Repo Doctor →
              </Link>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-base">Fingerprint</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2 text-sm">
            <div className="flex justify-between gap-4">
              <span className="text-zinc-400">Git HEAD</span>
              <span className="font-mono truncate">{repo.fingerprint.gitHead ?? "—"}</span>
            </div>
            <div className="flex justify-between gap-4">
              <span className="text-zinc-400">Working tree</span>
              <span className="font-mono truncate">{repo.fingerprint.workingTreeHash ?? "—"}</span>
            </div>
            <div className="flex justify-between gap-4">
              <span className="text-zinc-400">Schema</span>
              <Badge>{repo.fingerprint.schemaVersion ?? "none"}</Badge>
            </div>
            <div className="flex justify-between gap-4">
              <span className="text-zinc-400">Index version</span>
              <span>{repo.fingerprint.indexSchemaVersion}</span>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
