"use client";

import React, { useState, useEffect } from "react";
import { usePathname } from "next/navigation";
import { Sidebar } from "./Sidebar";
import { Header } from "./Header";
import { CommandPalette } from "@/components/common/CommandPalette";
import { LoginOverlay } from "@/components/common/LoginOverlay";

export function Shell({ children }: { children: React.ReactNode }) {
  const [mobileOpen, setMobileOpen] = useState(false);
  const [collapsed, setCollapsed] = useState(false);
  const pathname = usePathname();
  const isChat = pathname === "/chat";

  useEffect(() => {
    try {
      const saved = localStorage.getItem("ryvos_sidebar_collapsed");
      if (saved !== null) {
        setCollapsed(saved === "true");
      }
    } catch {}
  }, []);

  const handleToggleCollapse = () => {
    setCollapsed((prev) => {
      const next = !prev;
      try {
        localStorage.setItem("ryvos_sidebar_collapsed", String(next));
      } catch {}
      return next;
    });
  };

  return (
    <div className="neu-shell min-h-screen flex flex-col antialiased">
      <Sidebar
        mobileOpen={mobileOpen}
        onCloseMobile={() => setMobileOpen(false)}
        collapsed={collapsed}
        onToggleCollapse={handleToggleCollapse}
      />

      <div
        className={`flex-1 flex flex-col min-w-0 pb-0 transition-all duration-200 ${
          collapsed ? "md:pl-16" : "md:pl-60"
        }`}
      >
        <Header onOpenMobile={() => setMobileOpen(true)} />
        <main
          className={
            isChat
              ? "flex-1 flex flex-col h-[calc(100dvh-4rem)] overflow-hidden p-0 max-w-none"
              : "flex-1 p-4 md:p-8 max-w-7xl w-full mx-auto"
          }
        >
          {children}
        </main>
      </div>

      <CommandPalette />
      <LoginOverlay />
    </div>
  );
}
