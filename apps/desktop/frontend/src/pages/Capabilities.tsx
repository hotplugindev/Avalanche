import { Link } from "react-router-dom";
import { useCapabilities } from "@/queries";
import { Card, CardContent, CardHeader, CardTitle, LoadingBlock, ErrorBlock, EmptyBlock, Badge } from "@/components/ui";

export function Capabilities() {
  const { data: caps, isLoading, error } = useCapabilities();

  if (isLoading) return <LoadingBlock />;
  if (error) return <ErrorBlock message={String(error)} />;
  if (!caps || caps.length === 0) return <EmptyBlock>No capabilities found.</EmptyBlock>;

  return (
    <div className="space-y-4">
      <h1 className="text-2xl font-bold">Capabilities</h1>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {caps.map((c) => (
          <Card key={c.requestPath}>
            <CardHeader className="pb-2">
              <div className="flex items-center justify-between">
                <CardTitle className="text-base">
                  {c.id.domain ? `${c.id.domain}.${c.id.name}` : c.id.name}
                </CardTitle>
                <div className="flex gap-1">
                  <Badge className="capitalize">{c.id.scope}</Badge>
                  <Badge
                    className={
                      c.active
                        ? "border-emerald-800 text-emerald-400"
                        : "border-zinc-700 text-zinc-500"
                    }
                  >
                    {c.active ? "active" : "inactive"}
                  </Badge>
                </div>
              </div>
            </CardHeader>
            <CardContent className="space-y-2 text-sm">
              <div>
                <div className="text-zinc-400 mb-1">Request path</div>
                <Link
                  to={`/provenance?option=${encodeURIComponent(c.requestPath)}`}
                  className="font-mono text-xs text-zinc-300 hover:text-zinc-100 hover:underline"
                >
                  {c.requestPath}
                </Link>
              </div>
              <div>
                <div className="text-zinc-400 mb-1">Implementation</div>
                <div className="font-mono text-xs">{c.modulePath || "—"}</div>
              </div>
              {c.ownershipStatus && (
                <div className="flex justify-between gap-4">
                  <span className="text-zinc-400">Ownership</span>
                  <Badge className="capitalize">{c.ownershipStatus}</Badge>
                </div>
              )}
              <div>
                <div className="text-zinc-400 mb-1">Requesters ({c.requesters.length})</div>
                <div className="flex flex-wrap gap-1">
                  {c.requesters.length === 0 && <span className="text-zinc-600">none</span>}
                  {c.requesters.map((r) => (
                    <Badge key={r}>{r}</Badge>
                  ))}
                </div>
              </div>
              <div>
                <div className="text-zinc-400 mb-1">Owned options ({c.ownedOptions.length})</div>
                <div className="max-h-32 overflow-auto space-y-0.5">
                  {c.ownedOptions.slice(0, 30).map((o) => (
                    <Link
                      key={o}
                      to={`/provenance?option=${encodeURIComponent(o)}`}
                      className="block font-mono text-xs text-zinc-300 hover:text-zinc-100 hover:underline"
                    >
                      {o}
                    </Link>
                  ))}
                  {c.ownedOptions.length > 30 && (
                    <div className="text-xs text-zinc-600">…and {c.ownedOptions.length - 30} more</div>
                  )}
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}
