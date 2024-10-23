// app/page.tsx
"use client";

import { useSession } from "next-auth/react";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useEffect, useState } from "react";
import { useProfile, Repository } from "@/contexts/ProfileContext";
import { fetchWrapper } from "@/lib/fetchWrapper";
import { RepositoryCard } from "@/components/RepositoryCard";

function Main() {
  const [repositories, setRepositories] = useState<Repository[]>([]);
  const [activeTab, setActiveTab] = useState("recent");
  const { profile } = useProfile();

  const fetchRepositories = async () => {
    const response = await fetchWrapper("/api/repositories");
    if (!response.ok) throw new Error("Failed to fetch repositories");
    const data = await response.json();
    setRepositories(data);
  };

  useEffect(() => {
    fetchRepositories();
  }, []);

  const handleCollectionUpdate = () => {
    fetchRepositories();
  };

  return (
    <div className="w-full max-w-6xl mx-auto">
      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList>
          <TabsTrigger value="recent">Recent</TabsTrigger>
          <TabsTrigger value="search">Search</TabsTrigger>
        </TabsList>
        <TabsContent value="recent">
          <h1 className="text-2xl font-bold mb-4">Recent Repositories</h1>
          <p className="text-muted-foreground mb-4">
            These repositories have recently been added to devgrowth.
          </p>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-2 gap-4">
            {repositories.map((repo) => (
              <RepositoryCard
                key={repo.repository_id}
                repo={repo}
                collections={profile?.collections || []}
                onCollectionUpdate={handleCollectionUpdate}
              />
            ))}
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
}

export default function Home() {
  const { data: session, status } = useSession();
  const { profile, refetchProfile } = useProfile();

  useEffect(() => {
    if (status === "authenticated") {
      refetchProfile();
    }
  }, [status, refetchProfile]);

  if (status === "loading") {
    return <div>Loading...</div>;
  }
  return (
    <>
      {!session ? (
        <div className="text-center mt-4">
          <p className="mb-4">
            Please log in to view your starred repositories and collections
          </p>
          <Button asChild>
            <Link href="/api/auth/signin">Sign In</Link>
          </Button>
        </div>
      ) : profile ? (
        <Main />
      ) : (
        <div>Loading profile data...</div>
      )}
    </>
  );
}
