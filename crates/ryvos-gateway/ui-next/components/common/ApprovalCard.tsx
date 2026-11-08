"use client";

import React, { useState } from "react";
import { Icons } from "@/components/icons/Icons";
import { api } from "@/lib/api";
import type { ApprovalRequest } from "@/types/api";

export function ApprovalCard({
  approval,
  onResolved,
}: {
  approval: ApprovalRequest;
  onResolved: () => void;
}) {
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState("");

  const handleApprove = async () => {
    setSubmitting(true);
    setError("");
    try {
      await api.approvals.approve(approval.id);
      onResolved();
    } catch (e: any) {
      setError(e.message || "Gagal menyetujui");
    } finally {
      setSubmitting(false);
    }
  };

  const handleDeny = async () => {
    setSubmitting(true);
    setError("");
    try {
      await api.approvals.deny(approval.id);
      onResolved();
    } catch (e: any) {
      setError(e.message || "Gagal menolak");
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="p-4 my-2 rounded-2xl border border-warning/40 bg-warning/5 text-xs shadow-raised-sm">
      <div className="flex items-center gap-2 mb-2 text-warning font-semibold">
        <Icons.AlertCircle className="w-4 h-4" />
        <span>Persetujuan Tindakan (HITL)</span>
      </div>

      <div className="mb-2">
        <span className="text-text-secondary">Tool: </span>
        <span className="font-mono font-bold text-text">{approval.tool_name}</span>
      </div>

      <div className="mb-3">
        <div className="text-[10px] uppercase font-bold text-text-secondary mb-1">Payload</div>
        <pre className="p-2.5 rounded-xl bg-surface border border-border/60 text-text font-mono text-[11px] whitespace-pre-wrap max-h-36 overflow-y-auto">
          {approval.input}
        </pre>
      </div>

      {error && <div className="text-danger text-[11px] mb-2">{error}</div>}

      <div className="flex items-center gap-2">
        <button
          onClick={handleApprove}
          disabled={submitting}
          className="btn-primary !min-h-[32px] !py-1 !px-3 text-xs flex items-center gap-1.5"
        >
          <Icons.Check className="w-3.5 h-3.5" />
          <span>Setujui</span>
        </button>
        <button
          onClick={handleDeny}
          disabled={submitting}
          className="btn-danger !min-h-[32px] !py-1 !px-3 text-xs flex items-center gap-1.5"
        >
          <Icons.X className="w-3.5 h-3.5" />
          <span>Tolak</span>
        </button>
      </div>
    </div>
  );
}
