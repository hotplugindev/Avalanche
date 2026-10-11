import { useState } from "react";
import { useSearchParams } from "react-router-dom";
import { useProvenance, useSearch } from "@/queries";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  LoadingBlock,
  ErrorBlock,
  EmptyBlock,
  Badge,
  Input,
} from "@/components/ui";
import { cn } from "@/lib/utils";
import type { DefinitionView } from "@/types";

export function Provenance() {
  const [params, setParams] = useSearchParams();
  const option = params.get("option") ?? "";
  const [query, setQuery] = useState(option);
  const [prevOption, setPrevOption] = useState(option);
  const { data: hits } = useSearch(query);
  const { data: prov, isLoading, error } = useProvenance(option || undefined);

  if (option !== prevOption) {
    setPrevOption(option);
    setQuery(option);
  }

  const select = (path: string) => {
    setParams({ option: path });
  };

  return (
    <div className="space-y-4">
      <h1 className="text-2xl font-bold">Provenance</h1>

      <div className="relative max-w-xl">
        <Input
          placeholder="Search options…"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
        {query.length > 1 && hits && hits.length > 0 && !option && (
          <div className="absolute z-10 mt-1 w-full rounded-md border border-zinc-800 bg-zinc-900 shadow-lg max-h-72 overflow-auto">
            {hits.slice(0, 40).map((h) => (
              <button
                key={`${h.kind}-${h.name}`}
                onClick={() => select(h.name)}
                className="flex w-full items-center gap-2 px-3 py-2 text-left text-sm hover:bg-zinc-800"
              >
                <Badge className="capitalize shrink-0">{h.kind}</Badge>
                <span className="font-mono truncate">{h.name}</span>
              </button>
            ))}
          </div>
        )}
      </div>

      {!option && <EmptyBlock>Select an option to inspect its provenance.</EmptyBlock>}
      {isLoading && <LoadingBlock />}
      {error && <ErrorBlock message={String(error)} />}
      {prov && <ProvenanceDetail prov={prov} />}
    </div>
  );
}

function ProvenanceDetail({ prov }: { prov: NonNullable<ReturnType<typeof useProvenance>["data"]> }) {
  return (
    <div className="space-y-4">
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-base font-mono">{prov.optionPath}</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3 text-sm">
          <div className="flex items-center gap-3">
            <span className="text-zinc-400">Effective value</span>
            <code className="rounded bg-zinc-800 px-2 py-0.5 text-xs">
              {JSON.stringify(prov.effectiveValue ?? null)}
            </code>
          </div>
          {prov.winning && (
            <div className="flex items-center gap-3">
              <span className="text-zinc-400">Defined by</span>
              <DefinitionLine def={prov.winning} highlight />
            </div>
          )}
        </CardContent>
      </Card>

      {prov.definitions.length > 0 && (
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-base">All definitions ({prov.definitions.length})</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            {prov.definitions.map((d, i) => (
              <DefinitionRow
                key={i}
                def={d}
                isWinner={
                  prov.winning != null &&
                  prov.winning.file === d.file &&
                  prov.winning.line === d.line &&
                  prov.winning.column === d.column
                }
              />
            ))}
          </CardContent>
        </Card>
      )}

      {prov.contributions.length > 0 && (
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-base">
              Merged contributions ({prov.contributions.length})
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-1">
            {prov.contributions.map((c, i) => (
              <div key={i} className="flex items-center gap-2 text-xs">
                <code className="rounded bg-zinc-800 px-2 py-0.5 font-mono">
                  {JSON.stringify(c.value)}
                </code>
                {c.file && (
                  <span className="text-zinc-500 font-mono">
                    {c.file}:{c.line}
                  </span>
                )}
                {c.layer && <Badge className="capitalize">{c.layer}</Badge>}
                {c.priority != null && c.priority >= 1500 && (
                  <Badge className="border-blue-800 text-blue-300">default</Badge>
                )}
              </div>
            ))}
          </CardContent>
        </Card>
      )}
    </div>
  );
}

function DefinitionLine({ def, highlight }: { def: DefinitionView; highlight?: boolean }) {
  return (
    <span
      className={cn(
        "font-mono text-xs",
        highlight ? "text-emerald-300" : "text-zinc-300"
      )}
    >
      {def.file}:{def.line}
    </span>
  );
}

function DefinitionRow({ def, isWinner }: { def: DefinitionView; isWinner: boolean }) {
  return (
    <div
      className={cn(
        "rounded-md border p-3 text-sm",
        isWinner ? "border-emerald-900 bg-emerald-950/20" : "border-zinc-800"
      )}
    >
      <div className="flex items-center justify-between gap-2 flex-wrap">
        <div className="flex items-center gap-2">
          <DefinitionLine def={def} highlight={isWinner} />
          {isWinner && <Badge className="border-emerald-800 text-emerald-400">effective</Badge>}
        </div>
        <div className="flex items-center gap-1 flex-wrap">
          <Badge className="capitalize">{def.layer}</Badge>
          <Badge className="capitalize">{def.definitionType}</Badge>
          {def.isDefault && <Badge className="border-blue-800 text-blue-300">mkDefault</Badge>}
          <Badge>prio {def.priority}</Badge>
        </div>
      </div>
      <div className="mt-2 flex items-center gap-2 flex-wrap text-xs">
        <span className="text-zinc-400">value</span>
        <code className="rounded bg-zinc-800 px-2 py-0.5 font-mono">
          {JSON.stringify(def.value ?? null)}
        </code>
        {def.host && <Badge className="capitalize">host {def.host}</Badge>}
        {def.user && <Badge className="capitalize">user {def.user}</Badge>}
        {def.profile && <Badge className="capitalize">profile {def.profile}</Badge>}
      </div>
      {def.condition && (
        <div className="mt-2 text-xs">
          <span className="text-zinc-400">condition </span>
          <code className="font-mono text-amber-300">{def.condition.expression}</code>
          {def.condition.satisfied != null && (
            <Badge
              className={cn(
                "ml-2",
                def.condition.satisfied
                  ? "border-emerald-800 text-emerald-400"
                  : "border-red-800 text-red-400"
              )}
            >
              {def.condition.satisfied ? "satisfied" : "not satisfied"}
            </Badge>
          )}
        </div>
      )}
    </div>
  );
}
