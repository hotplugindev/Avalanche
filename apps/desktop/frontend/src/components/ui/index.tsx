import * as React from "react";
import { cn } from "@/lib/utils";

export function Card({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return <div className={cn("rounded-lg border border-zinc-800 bg-zinc-900/50", className)} {...props} />;
}

export function CardHeader({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return <div className={cn("flex flex-col space-y-1.5 p-6", className)} {...props} />;
}

export function CardTitle({ className, ...props }: React.HTMLAttributes<HTMLHeadingElement>) {
  return <h3 className={cn("text-lg font-semibold leading-none tracking-tight", className)} {...props} />;
}

export function CardContent({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return <div className={cn("p-6 pt-0", className)} {...props} />;
}

export function Badge({ className, ...props }: React.HTMLAttributes<HTMLSpanElement>) {
  return (
    <span
      className={cn(
        "inline-flex items-center rounded-md border border-zinc-700 px-2 py-0.5 text-xs font-medium",
        className
      )}
      {...props}
    />
  );
}

export function Button({ className, ...props }: React.ButtonHTMLAttributes<HTMLButtonElement>) {
  return (
    <button
      className={cn(
        "inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors",
        "border border-zinc-700 bg-zinc-800 hover:bg-zinc-700 text-zinc-100",
        "disabled:pointer-events-none disabled:opacity-50",
        "h-9 px-4 py-2",
        className
      )}
      {...props}
    />
  );
}

export function Input({ className, ...props }: React.InputHTMLAttributes<HTMLInputElement>) {
  return (
    <input
      className={cn(
        "flex h-9 w-full rounded-md border border-zinc-700 bg-zinc-900 px-3 py-1 text-sm",
        "text-zinc-100 placeholder:text-zinc-500 focus:outline-none focus:ring-1 focus:ring-zinc-500",
        className
      )}
      {...props}
    />
  );
}

export function Spinner() {
  return <div className="animate-spin h-4 w-4 border-2 border-zinc-600 border-t-zinc-200 rounded-full" />;
}

export function LoadingBlock() {
  return (
    <div className="flex items-center justify-center py-12 text-zinc-500">
      <Spinner />
      <span className="ml-3">Loading…</span>
    </div>
  );
}

export function ErrorBlock({ message }: { message: string }) {
  return (
    <div className="rounded-md border border-red-900 bg-red-950/40 p-4 text-sm text-red-300">
      {message}
    </div>
  );
}

export function EmptyBlock({ children }: { children: React.ReactNode }) {
  return <div className="py-12 text-center text-sm text-zinc-500">{children}</div>;
}
