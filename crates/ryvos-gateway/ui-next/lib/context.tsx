"use client";

import React, { createContext, useContext, useEffect, useState } from "react";
import { getApiKey, setApiKey, clearApiKey } from "./api";
import { wsClient } from "./ws";
import type { ActivityEvent, ModelInfo } from "@/types/api";

interface AuthContextType {
  token: string;
  isAuthenticated: boolean;
  login: (token: string) => void;
  logout: () => void;
}

const AuthContext = createContext<AuthContextType>({
  token: "",
  isAuthenticated: false,
  login: () => {},
  logout: () => {},
});

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [token, setToken] = useState("");
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
    const stored = getApiKey();
    if (stored) {
      setToken(stored);
      setIsAuthenticated(true);
      wsClient.connect(stored);
    } else {
      wsClient.connect();
    }

    const onUnauthorized = () => {
      setToken("");
      setIsAuthenticated(false);
    };
    window.addEventListener("ryvos:unauthorized", onUnauthorized);
    return () => window.removeEventListener("ryvos:unauthorized", onUnauthorized);
  }, []);

  const login = (newToken: string) => {
    setApiKey(newToken);
    setToken(newToken);
    setIsAuthenticated(true);
    wsClient.connect(newToken);
  };

  const logout = () => {
    clearApiKey();
    setToken("");
    setIsAuthenticated(false);
    wsClient.disconnect();
    wsClient.connect();
  };

  return (
    <AuthContext.Provider value={{ token, isAuthenticated, login, logout }}>
      {children}
    </AuthContext.Provider>
  );
}

export const useAuth = () => useContext(AuthContext);

interface ThemeContextType {
  theme: "light" | "dark";
  toggleTheme: () => void;
}

const ThemeContext = createContext<ThemeContextType>({
  theme: "light",
  toggleTheme: () => {},
});

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  const [theme, setTheme] = useState<"light" | "dark">("light");

  useEffect(() => {
    const saved = localStorage.getItem("ryvos_theme") as "light" | "dark" | null;
    if (saved) {
      setTheme(saved);
      document.documentElement.setAttribute("data-theme", saved);
    } else {
      const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
      const initial = prefersDark ? "dark" : "light";
      setTheme(initial);
      document.documentElement.setAttribute("data-theme", initial);
    }
  }, []);

  const toggleTheme = () => {
    const next = theme === "light" ? "dark" : "light";
    setTheme(next);
    localStorage.setItem("ryvos_theme", next);
    document.documentElement.setAttribute("data-theme", next);
  };

  return (
    <ThemeContext.Provider value={{ theme, toggleTheme }}>
      {children}
    </ThemeContext.Provider>
  );
}

export const useTheme = () => useContext(ThemeContext);

interface WebSocketContextType {
  status: "connected" | "disconnected" | "error";
  streamingText: string;
  activityFeed: ActivityEvent[];
  modelInfo: ModelInfo;
  sendWs: (method: string, params?: Record<string, any>) => Promise<any>;
  resetStreamingText: () => void;
}

const WebSocketContext = createContext<WebSocketContextType>({
  status: "disconnected",
  streamingText: "",
  activityFeed: [],
  modelInfo: { provider: "gemini", model_id: "gemini-3.6-flash", key_count: 0 },
  sendWs: async () => {},
  resetStreamingText: () => {},
});

export function WebSocketProvider({ children }: { children: React.ReactNode }) {
  const [status, setStatus] = useState(wsClient.status);
  const [streamingText, setStreamingText] = useState(wsClient.streamingText);
  const [activityFeed, setActivityFeed] = useState<ActivityEvent[]>(wsClient.activityFeed);
  const [modelInfo, setModelInfo] = useState<ModelInfo>(wsClient.modelInfo);

  useEffect(() => {
    const unsubStatus = wsClient.onStatus(setStatus);
    const unsubStream = wsClient.onStream(setStreamingText);
    const unsubFeed = wsClient.onFeed(setActivityFeed);
    const unsubModel = wsClient.onModel(setModelInfo);

    return () => {
      unsubStatus();
      unsubStream();
      unsubFeed();
      unsubModel();
    };
  }, []);

  return (
    <WebSocketContext.Provider
      value={{
        status,
        streamingText,
        activityFeed,
        modelInfo,
        sendWs: (method, params) => wsClient.sendWs(method, params),
        resetStreamingText: () => wsClient.resetStreamingText(),
      }}
    >
      {children}
    </WebSocketContext.Provider>
  );
}

export const useWebSocket = () => useContext(WebSocketContext);
