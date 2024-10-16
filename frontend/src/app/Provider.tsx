"use client";

import { SessionProvider } from "next-auth/react";
import { ProfileProvider } from "@/contexts/ProfileContext";

export function Provider({ children }: { children: React.ReactNode }) {
  return (
    <SessionProvider>
      <ProfileProvider>{children}</ProfileProvider>
    </SessionProvider>
  );
}
