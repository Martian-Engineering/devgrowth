"use client";

import Link from "next/link";
import Image from "next/image";
import { useSession, signIn, signOut } from "next-auth/react";
import { Button } from "@/components/ui/button";
import { useState } from "react";

export function Navbar() {
  const { data: session, status } = useSession();
  const [error, setError] = useState("");

  const handleSignIn = async () => {
    try {
      await signIn("github");
    } catch (error) {
      setError("Failed to sign in. Please try again.");
      console.error("Sign-in error:", error);
    }
  };

  const handleSignOut = async () => {
    await signOut({ redirect: false });
    await fetch("/api/auth/logout", { method: "POST" });
    window.location.href = "/";
  };

  return (
    <nav className="bg-background border-b">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex justify-between h-16">
          <div className="flex">
            <div className="flex-shrink-0 flex items-center">
              <Link href="/" className="text-2xl font-bold text-primary">
                DevGrowth
              </Link>
            </div>
            <div className="hidden sm:ml-6 sm:flex sm:space-x-8">
              <Link
                href="/"
                className="inline-flex items-center px-1 pt-1 text-sm font-medium text-primary"
              >
                Home
              </Link>
              {status === "authenticated" && (
                <Link
                  href="/repositories"
                  className="inline-flex items-center px-1 pt-1 text-sm font-medium text-primary"
                >
                  Repositories
                </Link>
              )}
            </div>
          </div>
          <div className="hidden sm:ml-6 sm:flex sm:items-center">
            {status === "authenticated" && session ? (
              <>
                <span className="text-sm text-primary mr-4">
                  Signed in as {session.user?.name}
                </span>
                {session.user?.image && (
                  <Image
                    src={session.user.image}
                    alt={`${session.user.name}'s avatar`}
                    width={32}
                    height={32}
                    className="rounded-full mr-4"
                  />
                )}
                <Button onClick={() => handleSignOut()} variant="outline">
                  Sign out
                </Button>
              </>
            ) : status === "unauthenticated" ? (
              <Button onClick={handleSignIn} variant="default">
                Sign in with GitHub
              </Button>
            ) : (
              <span>Loading...</span>
            )}
            {error && <p className="text-red-500">{error}</p>}
          </div>
        </div>
      </div>
    </nav>
  );
}
