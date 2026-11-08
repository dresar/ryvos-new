"use client";

import React from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { useWebSocket, useAuth } from "@/lib/context";
import { Icons } from "@/components/icons/Icons";

const NAV_LINKS = [
  { path: "/chat", label: "Chat", icon: Icons.Chat },
  { path: "/dashboard", label: "Dashboard", icon: Icons.Dashboard },
  { path: "/sessions", label: "Sessions", icon: Icons.Sessions },
  { path: "/runs", label: "Runs", icon: Icons.Runs },
  { path: "/goals", label: "Goals", icon: Icons.Goals },
  { path: "/audit", label: "Audit Trail", icon: Icons.Audit },
  { path: "/costs", label: "Costs", icon: Icons.Costs },
  { path: "/integrations", label: "Integrations", icon: Icons.Integrations },
  { path: "/channels", label: "Channels", icon: Icons.Channels },
  { path: "/viking", label: "Viking Memory", icon: Icons.Viking },
  { path: "/cron", label: "Graph & Cron", icon: Icons.Graph },
  { path: "/config", label: "Config", icon: Icons.Config },
  { path: "/settings", label: "Settings", icon: Icons.Settings },
];

export function Sidebar({
  mobileOpen,
  onCloseMobile,
  collapsed,
  onToggleCollapse,
}: {
  mobileOpen: boolean;
  onCloseMobile: () => void;
  collapsed: boolean;
  onToggleCollapse: () => void;
}) {
  const pathname = usePathname();
  const { status } = useWebSocket();
  const { logout } = useAuth();

  const getStatusColor = () => {
    if (status === "connected") return "bg-success";
    if (status === "error") return "bg-danger";
    return "bg-warning";
  };

  return (
    <>
      {mobileOpen && (
        <div
          onClick={onCloseMobile}
          className="fixed inset-0 z-40 bg-bg/60 backdrop-blur-sm md:hidden"
        />
      )}

      <aside
        className={`fixed top-0 bottom-0 left-0 z-40 bg-surface/85 backdrop-blur-md flex flex-col border-r border-border/60 transition-all duration-200 ease-in-out md:translate-x-0 ${
          mobileOpen ? "translate-x-0 w-60" : "-translate-x-full md:translate-x-0"
        } ${collapsed ? "md:w-16" : "md:w-60"}`}
      >
        <div className={`h-16 flex items-center border-b border-border/60 px-3 ${
          collapsed ? "justify-center" : "justify-between"
        }`}>
          <Link
            href="/dashboard"
            className="flex items-center gap-3 min-w-0"
            title="Ryvos Control Plane"
          >
            <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-primary to-primary-strong flex items-center justify-center text-white font-extrabold text-sm shadow-raised-sm shrink-0">
              R
            </div>
            {!collapsed && (
              <div className="min-w-0">
                <div className="font-extrabold text-sm tracking-tight text-text truncate">Ryvos</div>
                <div className="text-[10px] text-text-secondary font-mono truncate">Control Plane</div>
              </div>
            )}
          </Link>

          {!collapsed && (
            <div className="flex items-center gap-1.5">
              <span className="text-[10px] font-mono px-1.5 py-0.5 rounded-full bg-surface-inset border border-border/60 text-text-secondary">
                v0.9.0
              </span>
              <button
                type="button"
                onClick={onToggleCollapse}
                title="Ciutkan sidebar"
                className="hidden md:flex p-1 rounded-md text-text-secondary hover:text-text hover:bg-surface-inset transition-colors"
              >
                <Icons.ChevronLeft className="w-4 h-4" />
              </button>
              <button
                type="button"
                onClick={onCloseMobile}
                className="p-1 md:hidden text-text-secondary hover:text-text"
              >
                <Icons.X className="w-5 h-5" />
              </button>
            </div>
          )}
        </div>

        <nav className="flex-1 overflow-y-auto p-2 space-y-1 no-scrollbar">
          {NAV_LINKS.map((item) => {
            const Icon = item.icon;
            const active = pathname === item.path || (item.path !== "/" && pathname?.startsWith(item.path));
            return (
              <Link
                key={item.path}
                href={item.path}
                onClick={onCloseMobile}
                title={collapsed ? item.label : undefined}
                className={`flex items-center rounded-xl text-xs transition-all ${
                  collapsed
                    ? "justify-center px-0 py-2.5"
                    : "gap-3 px-3 py-2.5"
                } ${
                  active
                    ? "bg-primary-soft text-primary font-bold shadow-raised-sm"
                    : "text-text-secondary hover:text-text hover:bg-surface-inset font-medium"
                }`}
              >
                <Icon className={`w-4 h-4 shrink-0 ${active ? "text-primary" : "text-text-secondary"}`} />
                {!collapsed && <span className="truncate">{item.label}</span>}
              </Link>
            );
          })}
        </nav>

        {collapsed && (
          <div className="p-2 border-t border-border/60 hidden md:flex justify-center">
            <button
              type="button"
              onClick={onToggleCollapse}
              title="Perlebar sidebar"
              className="p-2 rounded-lg text-text-secondary hover:text-text hover:bg-surface-inset transition-colors"
            >
              <Icons.ChevronRight className="w-4 h-4" />
            </button>
          </div>
        )}

        <div className={`p-3 border-t border-border/60 flex items-center bg-surface-inset/30 ${
          collapsed ? "flex-col gap-3 justify-center" : "justify-between"
        }`}>
          <div className="flex items-center gap-2" title={`Status: ${status}`}>
            <span className={`w-2 h-2 rounded-full shrink-0 ${getStatusColor()}`} />
            {!collapsed && (
              <span className="text-[11px] font-mono capitalize text-text-secondary truncate">
                {status}
              </span>
            )}
          </div>

          <button
            type="button"
            onClick={logout}
            title="Keluar"
            className="p-1.5 rounded-md hover:bg-danger/10 text-text-secondary hover:text-danger transition-colors shrink-0"
          >
            <Icons.LogOut className="w-4 h-4" />
          </button>
        </div>
      </aside>
    </>
  );
}
