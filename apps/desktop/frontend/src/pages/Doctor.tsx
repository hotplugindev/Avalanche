import { useDoctor } from "@/queries";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  LoadingBlock,
  ErrorBlock,
  EmptyBlock,
  Badge,
} from "@/components/ui";
import { cn } from "@/lib/utils";

export function Doctor() {
  const { data: doctor, isLoading, error } = useDoctor();

  if (isLoading) return <LoadingBlock />;
  if (error) return <ErrorBlock message={String(error)} />;
  if (!doctor) return null;

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">Repo Doctor</h1>
        <div className="flex gap-2">
          <Badge
            className={cn(
              doctor.errorCount > 0
                ? "border-red-800 text-red-400"
                : "border-emerald-800 text-emerald-400"
            )}
          >
            {doctor.errorCount} errors
          </Badge>
          <Badge
            className={cn(
              doctor.warningCount > 0
                ? "border-amber-800 text-amber-400"
                : "border-zinc-700 text-zinc-500"
            )}
          >
            {doctor.warningCount} warnings
          </Badge>
        </div>
      </div>

      {doctor.findings.length === 0 ? (
        <EmptyBlock>No findings. The repository looks healthy.</EmptyBlock>
      ) : (
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-base">Findings ({doctor.findings.length})</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            {doctor.findings.map((f, i) => (
              <div
                key={i}
                className={cn(
                  "rounded-md border p-3 text-sm",
                  f.severity === "error"
                    ? "border-red-900 bg-red-950/20"
                    : f.severity === "warning"
                      ? "border-amber-900 bg-amber-950/20"
                      : "border-zinc-800"
                )}
              >
                <div className="flex items-start gap-2">
                  <Badge
                    className={cn(
                      "capitalize shrink-0 mt-0.5",
                      f.severity === "error"
                        ? "border-red-800 text-red-400"
                        : f.severity === "warning"
                          ? "border-amber-800 text-amber-400"
                          : "border-zinc-700 text-zinc-400"
                    )}
                  >
                    {f.severity}
                  </Badge>
                  <div className="min-w-0">
                    <div>{f.message}</div>
                    {(f.path || f.file) && (
                      <div className="mt-1 font-mono text-xs text-zinc-500">
                        {f.path ?? f.file}
                      </div>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </CardContent>
        </Card>
      )}

      {doctor.repairs.length > 0 && (
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-base">
              Proposed repairs ({doctor.repairs.length})
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            {doctor.repairs.map((r, i) => (
              <div key={i} className="rounded-md border border-zinc-800 p-3 text-sm">
                <div className="flex items-start justify-between gap-2">
                  <div className="min-w-0">
                    <div>{r.description}</div>
                    <div className="mt-1 font-mono text-xs text-zinc-500">{r.action}</div>
                  </div>
                  <Badge className="shrink-0">{(r.confidence * 100).toFixed(0)}%</Badge>
                </div>
              </div>
            ))}
            <div className="pt-2 text-xs text-zinc-500">
              Repairs are proposals only. Avalanche never applies them automatically.
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
