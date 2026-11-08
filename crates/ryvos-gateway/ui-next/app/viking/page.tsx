"use client";

import React, { useState, useEffect } from "react";
import { api } from "@/lib/api";
import { Icons } from "@/components/icons/Icons";
import type { VikingNode, VikingContent } from "@/types/api";

export default function VikingPage() {
  const [currentPath, setCurrentPath] = useState("");
  const [nodes, setNodes] = useState<VikingNode[]>([]);
  const [selectedContent, setSelectedContent] = useState<VikingContent | null>(null);
  const [level, setLevel] = useState<"L0" | "L1" | "L2">("L0");
  const [loading, setLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<VikingContent[] | null>(null);

  const loadDirectory = async (path: string) => {
    setLoading(true);
    setSearchResults(null);
    try {
      const data = await api.vikingList(path);
      setNodes(Array.isArray(data) ? data : []);
      setCurrentPath(path);
    } catch {} finally {
      setLoading(false);
    }
  };

  const loadContent = async (path: string, selectedLevel = level) => {
    try {
      const data = await api.vikingRead(path, selectedLevel);
      setSelectedContent(data);
    } catch {}
  };

  useEffect(() => {
    loadDirectory("");
  }, []);

  const handleSearch = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!searchQuery.trim()) {
      setSearchResults(null);
      return;
    }
    setLoading(true);
    try {
      const results = await api.vikingSearch(searchQuery.trim());
      setSearchResults(Array.isArray(results) ? results : []);
    } catch {} finally {
      setLoading(false);
    }
  };

  const pathParts = currentPath.split("/").filter(Boolean);

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 className="text-xl font-bold text-text">OpenViking Memory Browser</h2>
          <p className="text-xs text-text-secondary mt-0.5">
            Hierarchical multi-tier memory explorer (L0 Abstract, L1 Contextual, L2 Raw Knowledge)
          </p>
        </div>

        <button
          onClick={() => loadDirectory(currentPath)}
          className="btn-soft !min-h-[36px] !py-1 text-xs self-start sm:self-auto"
        >
          <Icons.Refresh className="w-3.5 h-3.5" />
          <span>Refresh</span>
        </button>
      </div>

      <div className="neu-card p-4">
        <form onSubmit={handleSearch} className="flex items-center gap-2">
          <div className="flex-1 neu-inset flex items-center gap-2 px-3 py-2 bg-surface-inset">
            <Icons.Search className="w-4 h-4 text-text-secondary" />
            <input
              type="text"
              placeholder="Search memory vector & keywords..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full bg-transparent text-xs text-text placeholder:text-text-secondary/50 focus:outline-none"
            />
          </div>
          <button
            type="submit"
            className="btn-primary !min-h-[38px] !py-1 !px-4 text-xs"
          >
            Search
          </button>
          {searchResults && (
            <button
              type="button"
              onClick={() => {
                setSearchResults(null);
                setSearchQuery("");
              }}
              className="btn-outline !min-h-[38px] !py-1 !px-3 text-xs"
            >
              Clear
            </button>
          )}
        </form>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="neu-focal p-4 flex flex-col h-[520px]">
          <div className="flex items-center gap-2 pb-3 border-b border-border text-xs overflow-x-auto no-scrollbar">
            <button
              onClick={() => loadDirectory("")}
              className={`font-semibold hover:text-primary ${currentPath === "" ? "text-primary" : "text-text-secondary"}`}
            >
              root
            </button>
            {pathParts.map((part, i) => {
              const full = pathParts.slice(0, i + 1).join("/");
              return (
                <React.Fragment key={full}>
                  <span className="text-text-secondary">/</span>
                  <button
                    onClick={() => loadDirectory(full)}
                    className="hover:text-primary text-text-secondary font-medium whitespace-nowrap"
                  >
                    {part}
                  </button>
                </React.Fragment>
              );
            })}
          </div>

          <div className="flex-1 overflow-y-auto mt-3 space-y-1 pr-1 no-scrollbar">
            {loading ? (
              <div className="p-8 text-center text-xs text-text-secondary">Loading memory tree...</div>
            ) : searchResults ? (
              searchResults.length === 0 ? (
                <div className="p-8 text-center text-xs text-text-secondary">No matching memories</div>
              ) : (
                (Array.isArray(searchResults) ? searchResults : []).map((res, idx) => (
                  <button
                    key={idx}
                    onClick={() => setSelectedContent(res)}
                    className="w-full p-2.5 rounded-lg text-left bg-surface hover:bg-surface-inset border border-border text-xs"
                  >
                    <div className="font-mono text-primary font-bold truncate">{res.path}</div>
                    <div className="text-[10px] text-text-secondary truncate mt-0.5">{res.summary || res.content}</div>
                  </button>
                ))
              )
            ) : nodes.length === 0 ? (
              <div className="p-8 text-center text-xs text-text-secondary">Directory empty</div>
            ) : (
              (Array.isArray(nodes) ? nodes : []).map((node) => (
                <button
                  key={node.path}
                  onClick={() => {
                    if (node.is_dir) {
                      loadDirectory(node.path);
                    } else {
                      loadContent(node.path);
                    }
                  }}
                  className="w-full flex items-center justify-between p-2 rounded-md hover:bg-surface-inset text-xs text-left text-text transition-colors"
                >
                  <div className="flex items-center gap-2 truncate">
                    <span className="text-primary font-bold">
                      {node.is_dir ? "📁" : "📄"}
                    </span>
                    <span className="font-mono truncate">{node.name}</span>
                  </div>
                  {node.size !== undefined && (
                    <span className="text-[10px] text-text-secondary font-mono">{node.size}B</span>
                  )}
                </button>
              ))
            )}
          </div>
        </div>

        <div className="lg:col-span-2 neu-focal p-5 flex flex-col h-[520px]">
          <div className="flex items-center justify-between pb-3 border-b border-border">
            <div className="flex items-center gap-2">
              <span className="text-xs font-bold text-text truncate">
                {selectedContent?.path || "Select a memory node"}
              </span>
            </div>

            <div className="flex items-center gap-1">
              {(["L0", "L1", "L2"] as const).map((lvl) => (
                <button
                  key={lvl}
                  onClick={() => {
                    setLevel(lvl);
                    if (selectedContent?.path) loadContent(selectedContent.path, lvl);
                  }}
                  className={`px-2.5 py-1 rounded text-[11px] font-bold font-mono transition-all ${
                    level === lvl
                      ? "bg-primary text-white"
                      : "bg-surface-inset text-text-secondary hover:text-text"
                  }`}
                >
                  {lvl}
                </button>
              ))}
            </div>
          </div>

          <div className="flex-1 overflow-y-auto mt-4 p-4 rounded-xl bg-surface-inset border border-border">
            {selectedContent ? (
              <div className="space-y-3 font-mono text-xs">
                {selectedContent.summary && (
                  <div className="p-3 rounded bg-surface border border-border text-primary leading-relaxed font-sans text-xs">
                    <div className="font-bold uppercase text-[10px] text-text-secondary mb-1">
                      L0 Semantic Summary
                    </div>
                    {selectedContent.summary}
                  </div>
                )}
                <pre className="text-text whitespace-pre-wrap leading-relaxed">
                  {selectedContent.content}
                </pre>
              </div>
            ) : (
              <div className="flex flex-col items-center justify-center h-full text-center text-text-secondary text-xs">
                <Icons.Viking className="w-8 h-8 text-primary/40 mb-2" />
                <span>Select a file from the Viking hierarchy to inspect memory contents</span>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
