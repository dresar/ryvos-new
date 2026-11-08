import type { Metadata } from "next";
import "./globals.css";
import { AuthProvider, ThemeProvider, WebSocketProvider } from "@/lib/context";
import { Shell } from "@/components/layout/Shell";

export const metadata: Metadata = {
  title: "Ryvos Control Plane",
  description: "Autonomous AI Coding Agent Control Plane",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" data-theme="light">
      <body>
        <ThemeProvider>
          <AuthProvider>
            <WebSocketProvider>
              <Shell>{children}</Shell>
            </WebSocketProvider>
          </AuthProvider>
        </ThemeProvider>
      </body>
    </html>
  );
}
