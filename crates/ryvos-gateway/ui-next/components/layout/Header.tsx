"use client";

import React from "react";
import { usePathname } from "next/navigation";
import { useTheme } from "@/lib/context";
import { Icons } from "@/components/icons/Icons";

const PAGE_TITLES: Record<string, string> = {
  "/chat": "Chat",
  "/dashboard": "Dashboard",
  "/sessions": "Sesi",
  "/runs": "Runs",
  "/goals": "Goals",
  "/audit": "Audit",
  "/costs": "Biaya & Token",
  "/integrations": "Integrasi",
  "/channels": "Channels",
  "/viking": "Viking Memory",
  "/cron": "Cron & Jobs",
  "/config": "Konfigurasi",
  "/settings": "Pengaturan",
};

export function Header({ onOpenMobile }: { onOpenMobile: () => void }) {
  const pathname = usePathname();
  const { theme, toggleTheme } = useTheme();

  const title = PAGE_TITLES[pathname] || "Ryvos AI";

  return (
    <header className="sticky top-0 z-30 h-16 border-b border-border/60 bg-surface/80 backdrop-blur-md px-4 md:px-8 flex items-center justify-between">
      <div className="flex items-center gap-3">
        <button
          onClick={onOpenMobile}
          className="p-2 md:hidden text-text-secondary hover:text-text rounded-md hover:bg-surface-inset"
        >
          <Icons.Menu className="w-5 h-5" />
        </button>
        <h1 className="text-base font-bold text-text truncate">{title}</h1>
      </div>

      <div className="flex items-center gap-3">
        <button
          onClick={toggleTheme}
          title="Ganti Tema"
          className="p-2 rounded-lg neu-inset text-text-secondary hover:text-text transition-colors border border-border/60"
        >
          {theme === "light" ? (
            <Icons.Moon className="w-4 h-4" />
          ) : (
            <Icons.Sun className="w-4 h-4 text-warning" />
          )}
        </button>
      </div>
    </header>
  );
}
