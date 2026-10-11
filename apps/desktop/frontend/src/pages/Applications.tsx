import { Link } from "react-router-dom";
import { useApplications } from "@/queries";
import { Card, CardContent, CardHeader, CardTitle, LoadingBlock, ErrorBlock, EmptyBlock, Badge } from "@/components/ui";

export function Applications() {
  const { data: apps, isLoading, error } = useApplications();

  if (isLoading) return <LoadingBlock />;
  if (error) return <ErrorBlock message={String(error)} />;
  if (!apps || apps.length === 0) return <EmptyBlock>No applications found.</EmptyBlock>;

  return (
    <div className="space-y-4">
      <h1 className="text-2xl font-bold">Applications</h1>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {apps.map((a) => (
          <Card key={a.optionPath}>
            <CardHeader className="pb-2">
              <div className="flex items-center justify-between">
                <CardTitle className="text-base">{a.id.name}</CardTitle>
                <Badge className="capitalize">{a.id.scope}</Badge>
              </div>
            </CardHeader>
            <CardContent className="space-y-2 text-sm">
              {a.id.category && (
                <div className="flex justify-between gap-4">
                  <span className="text-zinc-400">Category</span>
                  <span>{a.id.category}</span>
                </div>
              )}
              <div>
                <div className="text-zinc-400 mb-1">Option</div>
                <Link
                  to={`/provenance?option=${encodeURIComponent(a.optionPath)}`}
                  className="font-mono text-xs text-zinc-300 hover:text-zinc-100 hover:underline"
                >
                  {a.optionPath}
                </Link>
              </div>
              <div>
                <div className="text-zinc-400 mb-1">Module</div>
                <div className="font-mono text-xs">{a.modulePath || "—"}</div>
              </div>
              <div>
                <div className="text-zinc-400 mb-1">Requested capabilities</div>
                <div className="flex flex-wrap gap-1">
                  {a.requestedCapabilities.length === 0 && (
                    <span className="text-zinc-600">none</span>
                  )}
                  {a.requestedCapabilities.map((c) => (
                    <Badge key={c}>{c}</Badge>
                  ))}
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}
