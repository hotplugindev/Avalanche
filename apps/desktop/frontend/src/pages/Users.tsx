import { useUsers } from "@/queries";
import { Card, CardContent, CardHeader, CardTitle, LoadingBlock, ErrorBlock, EmptyBlock, Badge } from "@/components/ui";

export function Users() {
  const { data: users, isLoading, error } = useUsers();

  if (isLoading) return <LoadingBlock />;
  if (error) return <ErrorBlock message={String(error)} />;
  if (!users || users.length === 0) return <EmptyBlock>No users found.</EmptyBlock>;

  return (
    <div className="space-y-4">
      <h1 className="text-2xl font-bold">Users</h1>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {users.map((u) => (
          <Card key={u.username}>
            <CardHeader className="pb-2">
              <div className="flex items-center justify-between">
                <CardTitle className="text-base">{u.username}</CardTitle>
                {u.isMain && <Badge className="border-emerald-800 text-emerald-400">main</Badge>}
              </div>
            </CardHeader>
            <CardContent className="space-y-2 text-sm">
              <div className="flex justify-between gap-4">
                <span className="text-zinc-400">Full name</span>
                <span>{u.fullName || "—"}</span>
              </div>
              <div className="flex justify-between gap-4">
                <span className="text-zinc-400">Email</span>
                <span className="font-mono truncate">{u.email || "—"}</span>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}
