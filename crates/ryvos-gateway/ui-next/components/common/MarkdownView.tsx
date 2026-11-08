"use client";

import React, { useMemo } from "react";
import { marked } from "marked";
import DOMPurify from "dompurify";

export function MarkdownView({ content }: { content: string }) {
  const sanitizedHtml = useMemo(() => {
    if (!content) return "";
    try {
      const rawHtml = marked.parse(content, { async: false, breaks: true }) as string;
      if (typeof window !== "undefined") {
        return DOMPurify.sanitize(rawHtml);
      }
      return rawHtml;
    } catch {
      return content;
    }
  }, [content]);

  return (
    <div
      className="prose prose-sm max-w-none text-text text-xs leading-relaxed space-y-2 break-words"
      dangerouslySetInnerHTML={{ __html: sanitizedHtml }}
    />
  );
}
