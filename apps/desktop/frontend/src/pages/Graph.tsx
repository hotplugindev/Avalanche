import { useMemo, useState } from "react";
import { useGraph } from "@/queries";
import { LoadingBlock, ErrorBlock, EmptyBlock, Badge, Input } from "@/components/ui";
import { cn } from "@/lib/utils";

const kindColors: Record<string, string> = {
  profile: "border-purple-800 text-purple-300",
  capability: "border-blue-800 text-blue-300",
  program: "border-emerald-800 text-emerald-300",
  option: "border-zinc-700 text-zinc-300",
  module: "border-amber-800 text-amber-300",
  host: "border-rose-800 text-rose-300",
  user: "border-teal-800 text-teal-300",
  desktop: "border-indigo-800 text-indigo-300",
  aggregate: "border-orange-800 text-orange-300",
  core: "border-slate-700 text-slate-300",
};

export function Graph() {
  const { data: graph, isLoading, error } = useGraph();
  const [filter, setFilter] = useState("");

  const filtered = useMemo(() => {
    if (!graph) return null;
    const q = filter.toLowerCase();
    const nodes = graph.nodes.filter(
      (n) => !q || n.id.toLowerCase().includes(q) || n.label.toLowerCase().includes(q)
    );
    const ids = new Set(nodes.map((n) => n.id));
    const edges = graph.edges.filter((e) => ids.has(e.from) && ids.has(e.to));
    return { nodes, edges };
  }, [graph, filter]);

  if (isLoading) return <LoadingBlock />;
  if (error) return <ErrorBlock message={String(error)} />;
  if (!filtered || filtered.nodes.length === 0) return <EmptyBlock>No graph data.</EmptyBlock>;

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">Request Graph</h1>
        <div className="w-64">
          <Input
            placeholder="Filter nodes…"
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
          />
        </div>
      </div>

      <div className="flex flex-wrap gap-2 text-xs">
        {Object.entries(kindColors).map(([k, cls]) => (
          <Badge key={k} className={cn("capitalize", cls)}>{k}</Badge>
        ))}
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <div className="rounded-lg border border-zinc-800 p-4">
          <div className="text-sm font-medium text-zinc-400 mb-3">
            Nodes ({filtered.nodes.length})
          </div>
          <div className="max-h-[60vh] overflow-auto space-y-1">
            {filtered.nodes.map((n) => (
              <div
                key={n.id}
                className="flex items-center gap-2 text-sm"
              >
                <Badge className={cn("capitalize shrink-0", kindColors[n.kind] ?? "")}>{n.kind}</Badge>
                <span className="font-mono text-xs truncate" title={n.id}>{n.id}</span>
              </div>
            ))}
          </div>
        </div>

        <div className="rounded-lg border border-zinc-800 p-4">
          <div className="text-sm font-medium text-zinc-400 mb-3">
            Edges ({filtered.edges.length})
          </div>
          <div className="max-h-[60vh] overflow-auto space-y-1">
            {filtered.edges.map((e, i) => (
              <div key={i} className="text-xs font-mono flex items-center gap-1 flex-wrap">
                <span className="text-zinc-400 truncate max-w-[38%]">{e.from}</span>
                <Badge className="capitalize shrink-0">{e.kind}</Badge>
                <span className="text-zinc-200 truncate max-w-[38%]">{e.to}</span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
