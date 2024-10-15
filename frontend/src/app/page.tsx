// app/page.tsx
"use client";

import { useSession } from "next-auth/react";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { useState, useEffect } from "react";
import { StarredReposList } from "@/components/StarredReposList";
import { CollectionsList } from "@/components/CollectionsList";
import { fetchWrapper } from "@/lib/fetchWrapper";

interface RepoCollectionMap {
  [repoId: number]: number[];
}

interface ProfileData {
  github_id: string;
  login: string;
  name: string | null;
  email: string | null;
  starred_repositories: StarredRepo[];
  repo_collections: RepoCollectionMap;
}

interface StarredRepo {
  id: number;
  name: string;
  owner: string;
  html_url: string;
  description: string | null;
  stargazers_count: number | null;
  synced_at: Date | null;
}

export default function Home() {
  const { data: session, status } = useSession();
  const [profileData, setProfileData] = useState<ProfileData | null>(null);

  const fetchProfileData = () => {
    fetchWrapper("/api/account/profile", {
      credentials: "include",
    })
      .then((response) => {
        if (!response.ok) {
          throw new Error("Failed to fetch profile data");
        }
        return response.json();
      })
      .then(setProfileData)
      .catch((error) => console.error("Error fetching profile data:", error));
  };

  useEffect(() => {
    if (status === "authenticated") {
      // TODO: Implement fetch for starred repos and profile data (repoCollections) as separate endpoints
      // TODO: Optimistic update of UI
      fetchProfileData();
    }
  }, [status]);

  if (status === "loading") {
    return <div>Loading...</div>;
  }

  return (
    <main className="flex min-h-screen flex-col items-center p-24">
      {!session ? (
        <div className="text-center mt-4">
          <p className="mb-4">
            Please log in to view your starred repositories and collections
          </p>
          <Button asChild>
            <Link href="/api/auth/signin">Sign In</Link>
          </Button>
        </div>
      ) : profileData ? (
        <div className="w-full max-w-7xl">
          <h1 className="text-3xl font-semibold mb-8">
            Welcome, {profileData.name || profileData.login}!
          </h1>
          <div className="flex flex-col md:flex-row gap-8">
            <div className="w-full md:w-1/2">
              <StarredReposList
                repos={profileData.starred_repositories}
                repoCollections={profileData.repo_collections}
                onCollectionUpdate={fetchProfileData}
              />
            </div>
            <div className="w-full md:w-1/2">
              <CollectionsList />
            </div>
          </div>
        </div>
      ) : (
        <div>Loading profile data...</div>
      )}
    </main>
  );
}
