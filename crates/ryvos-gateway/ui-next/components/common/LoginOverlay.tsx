"use client";

import React, { useState } from "react";
import { useAuth } from "@/lib/context";
import { Icons } from "@/components/icons/Icons";

export function LoginOverlay() {
  const { isAuthenticated, login } = useAuth();
  const [tokenInput, setTokenInput] = useState("");
  const [submitting, setSubmitting] = useState(false);

  if (isAuthenticated) return null;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitting(true);
    login(tokenInput.trim());
    setSubmitting(false);
  };

  const handleBlank = () => {
    login("");
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-bg/80 backdrop-blur-md p-4">
      <div className="neu-focal p-8 w-full max-w-md relative overflow-hidden">
        <div className="flex flex-col items-center text-center mb-6">
          <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-primary to-primary-strong flex items-center justify-center text-white mb-3 shadow-raised-sm">
            <span className="font-extrabold text-xl tracking-wider">R</span>
          </div>
          <h2 className="text-xl font-bold text-text">Ryvos Control Plane</h2>
          <p className="text-xs text-text-secondary mt-1">
            Masukkan password gateway untuk menghubungkan sistem
          </p>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-xs font-semibold text-text-secondary mb-1">
              Password / Gateway Token
            </label>
            <input
              type="password"
              placeholder="Masukkan password..."
              value={tokenInput}
              onChange={(e) => setTokenInput(e.target.value)}
              className="neu-inset w-full px-3 py-2.5 text-sm font-mono placeholder:text-text-secondary/50 focus:outline-none"
            />
          </div>

          <button
            type="submit"
            disabled={submitting}
            className="btn-primary w-full shadow-raised-sm"
          >
            <span>Masuk ke Ryvos</span>
            <Icons.ChevronRight className="w-4 h-4" />
          </button>
        </form>

        <div className="mt-4 pt-4 border-t border-border text-center">
          <button
            onClick={handleBlank}
            type="button"
            className="text-xs text-text-secondary hover:text-primary transition-colors underline"
          >
            Masuk tanpa token (Mode Publik)
          </button>
        </div>
      </div>
    </div>
  );
}
