import React from "react";

export function MetricCard({
  label,
  value,
  sub,
  icon,
}: {
  label: string;
  value: string | number;
  sub?: string;
  icon?: React.ReactNode;
}) {
  return (
    <div className="neu-card p-4 sm:p-5 flex flex-col justify-between transition-all hover:translate-y-[-1px] border border-border/60 shadow-raised-sm">
      <div className="flex items-center justify-between text-text-secondary mb-2">
        <span className="text-xs font-bold tracking-wider uppercase">{label}</span>
        {icon && <div className="text-primary">{icon}</div>}
      </div>
      <div>
        <div className="text-2xl font-bold tabular-nums text-text">{value}</div>
        {sub && <div className="text-xs text-text-secondary mt-1">{sub}</div>}
      </div>
    </div>
  );
}
