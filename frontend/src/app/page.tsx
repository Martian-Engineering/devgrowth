// app/page.tsx
"use client";

import { useSession } from "next-auth/react";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { useState, useEffect } from "react";
import { StarredReposList } from "@/components/StarredReposList";
import { CollectionsList } from "@/components/CollectionsList";
import { fetchWrapper } from "@/lib/fetchWrapper";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { ReloadIcon } from "@radix-ui/react-icons";
import { useProfile } from "@/contexts/ProfileContext";

export default function Home() {
  const { data: session, status } = useSession();
  const { profile, refetchProfile, refetchCollections, dispatch } =
    useProfile();
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [newCollectionName, setNewCollectionName] = useState("");
  const [newCollectionDescription, setNewCollectionDescription] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleCreateCollection = async () => {
    setIsSubmitting(true);
    try {
      const response = await fetchWrapper("/api/collections", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          name: newCollectionName,
          description: newCollectionDescription,
        }),
      });

      if (!response.ok) {
        throw new Error("Failed to create collection");
      }

      dispatch({ type: "CREATE_COLLECTION", payload: await response.json() });

      // Close the dialog and refresh the data
      setIsDialogOpen(false);
      // TODO: Try with and without this
      refetchCollections();
    } catch (error) {
      console.error("Error creating collection:", error);
    } finally {
      setIsSubmitting(false);
      setNewCollectionName("");
      setNewCollectionDescription("");
    }
  };

  useEffect(() => {
    if (status === "authenticated") {
      refetchProfile();
    }
  }, [status, refetchProfile]);

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
      ) : profile ? (
        <div className="w-full max-w-7xl">
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
                <Button onClick={() => setIsDialogOpen(true)}>
                  Create New Collection
                </Button>
              </div>
              <CollectionsList
                collections={(profile && profile.collections) || []}
              />
            </div>
          </div>

          <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
            <DialogContent>
              <DialogHeader>
                <DialogTitle>Create New Collection</DialogTitle>
              </DialogHeader>
              <div className="grid gap-4 py-4">
                <div className="grid grid-cols-4 items-center gap-4">
                  <Label htmlFor="name" className="text-right">
                    Name
                  </Label>
                  <Input
                    id="name"
                    value={newCollectionName}
                    onChange={(e) => setNewCollectionName(e.target.value)}
                    className="col-span-3"
                  />
                </div>
                <div className="grid grid-cols-4 items-center gap-4">
                  <Label htmlFor="description" className="text-right">
                    Description
                  </Label>
                  <Textarea
                    id="description"
                    value={newCollectionDescription}
                    onChange={(e) =>
                      setNewCollectionDescription(e.target.value)
                    }
                    className="col-span-3"
                  />
                </div>
              </div>
              <DialogFooter>
                <Button
                  variant="outline"
                  onClick={() => setIsDialogOpen(false)}
                >
                  Cancel
                </Button>
                <Button
                  onClick={handleCreateCollection}
                  disabled={!newCollectionName || isSubmitting}
                >
                  {isSubmitting ? (
                    <>
                      <ReloadIcon className="mr-2 h-4 w-4 animate-spin" />
                      Creating...
                    </>
                  ) : (
                    "Create"
                  )}
                </Button>
              </DialogFooter>
            </DialogContent>
          </Dialog>
        </div>
      ) : (
        <div>Loading profile data...</div>
      )}
    </main>
  );
}
