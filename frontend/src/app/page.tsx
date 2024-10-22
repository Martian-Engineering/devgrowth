// app/page.tsx
"use client";

import { useSession } from "next-auth/react";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { useEffect } from "react";
import { StarredReposList } from "@/components/StarredReposList";
import { CollectionsList } from "@/components/CollectionsList";
import { useProfile } from "@/contexts/ProfileContext";

function Main() {
  const { profile, refetchCollections } = useProfile();

  return (
    <div className="w-full max-w-6xl">
      <h1 className="text-3xl font-semibold mb-8">
        Welcome,{" "}
        {profile &&
          profile.account &&
          (profile.account.name || profile.account.login)}
        !
      </h1>
      <div className="flex flex-col md:flex-row gap-8">
        <div className="w-full md:w-1/2">
          <StarredReposList
            repos={profile.starred_repositories || []}
            onCollectionUpdate={refetchCollections}
          />
        </div>
        <div className="w-full md:w-1/2">
          <div className="flex justify-between items-center mb-4">
            <h2 className="text-2xl font-semibold">Collections</h2>
          </div>
          <CollectionsList
            collections={(profile && profile.collections) || []}
          />
        </div>
      </div>
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
