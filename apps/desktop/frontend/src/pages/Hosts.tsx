import { useHosts } from "@/queries";
import { Card, CardContent, CardHeader, CardTitle, LoadingBlock, ErrorBlock, EmptyBlock, Badge } from "@/components/ui";

export function Hosts() {
  const { data: hosts, isLoading, error } = useHosts();

  if (isLoading) return <LoadingBlock />;
  if (error) return <ErrorBlock message={String(error)} />;
  if (!hosts || hosts.length === 0) return <EmptyBlock>No hosts found.</EmptyBlock>;

  return (
    <div className="space-y-4">
      <h1 className="text-2xl font-bold">Hosts</h1>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {hosts.map((h) => (
          <Card key={h.name}>
            <CardHeader className="pb-2">
              <div className="flex items-center justify-between">
                <CardTitle className="text-base">{h.name}</CardTitle>
                <Badge className="capitalize">{h.class}</Badge>
              </div>
            </CardHeader>
            <CardContent className="space-y-2 text-sm">
              <Row label="System" value={h.system} mono />
              <Row label="Desktop" value={h.desktop} />
              <Row label="Shell" value={h.shell} />
              <Row label="State version" value={h.stateVersion} />
              <div>
                <div className="text-zinc-400 mb-1">Roles</div>
                <div className="flex flex-wrap gap-1">
                  {h.roles.length === 0 && <span className="text-zinc-600">none</span>}
                  {h.roles.map((r) => (
                    <Badge key={r} className="capitalize">{r}</Badge>
                  ))}
                </div>
              </div>
              <div>
                <div className="text-zinc-400 mb-1">Hardware</div>
                <div className="grid grid-cols-2 gap-x-4 gap-y-1 text-xs">
                  <span>CPU: <span className="capitalize">{h.hardware.cpu}</span></span>
                  <span>GPU: <span className="capitalize">{h.hardware.gpu}</span></span>
                  <Fact label="battery" value={h.hardware.hasBattery} />
                  <Fact label="bluetooth" value={h.hardware.hasBluetooth} />
                  <Fact label="touchpad" value={h.hardware.hasTouchpad} />
                  <Fact label="printer" value={h.hardware.hasPrinter} />
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}

function Row({ label, value, mono }: { label: string; value: string; mono?: boolean }) {
  return (
    <div className="flex justify-between gap-4">
      <span className="text-zinc-400">{label}</span>
      <span className={mono ? "font-mono" : "capitalize"}>{value}</span>
    </div>
  );
}

function Fact({ label, value }: { label: string; value: boolean }) {
  return (
    <span className={value ? "text-emerald-400" : "text-zinc-600"}>
      {value ? "✓" : "✗"} {label}
    </span>
  );
}
