import { Link } from "react-router-dom";
import { useProfiles } from "@/queries";
import { Card, CardContent, CardHeader, CardTitle, LoadingBlock, ErrorBlock, EmptyBlock, Badge } from "@/components/ui";

export function Profiles() {
  const { data: profiles, isLoading, error } = useProfiles();

  if (isLoading) return <LoadingBlock />;
  if (error) return <ErrorBlock message={String(error)} />;
  if (!profiles || profiles.length === 0) return <EmptyBlock>No profiles found.</EmptyBlock>;

  return (
    <div className="space-y-4">
      <h1 className="text-2xl font-bold">Profiles</h1>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {profiles.map((p) => (
          <Card key={`${p.id.scope}-${p.id.name}`}>
            <CardHeader className="pb-2">
              <div className="flex items-center justify-between">
                <CardTitle className="text-base">{p.id.name}</CardTitle>
                <Badge className="capitalize">{p.id.scope}</Badge>
              </div>
            </CardHeader>
            <CardContent className="space-y-3 text-sm">
              <div>
                <div className="text-zinc-400 mb-1">Source</div>
                <div className="font-mono text-xs">{p.sourceFile}</div>
              </div>
              <div>
                <div className="text-zinc-400 mb-1">Requested capabilities</div>
                <div className="flex flex-wrap gap-1">
                  {p.requestedCapabilities.length === 0 && (
                    <span className="text-zinc-600">none</span>
                  )}
                  {p.requestedCapabilities.map((c) => (
                    <Badge key={c}>{c}</Badge>
                  ))}
                </div>
              </div>
              <div>
                <div className="text-zinc-400 mb-1">Default options ({p.defaultOptions.length})</div>
                <div className="max-h-40 overflow-auto space-y-0.5">
                  {p.defaultOptions.slice(0, 50).map((o) => (
                    <Link
                      key={o}
                      to={`/provenance?option=${encodeURIComponent(o)}`}
                      className="block font-mono text-xs text-zinc-300 hover:text-zinc-100 hover:underline"
                    >
                      {o}
                    </Link>
                  ))}
                  {p.defaultOptions.length > 50 && (
                    <div className="text-xs text-zinc-600">…and {p.defaultOptions.length - 50} more</div>
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
