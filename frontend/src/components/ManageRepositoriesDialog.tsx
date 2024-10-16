// devgrowth/frontend/src/components/ManageRepositoriesDialog.tsx
import React, { useState, useEffect } from "react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { ManageRepositoriesTable } from "@/components/ManageRepositoriesTable";
import { Button } from "@/components/ui/button";
import { DialogFooter } from "@/components/ui/dialog";
import { fetchWrapper } from "@/lib/fetchWrapper";
import { GithubRepo, useProfile } from "@/contexts/ProfileContext";
import { toast } from "@/hooks/use-toast";

interface ManageRepositoriesDialogProps {
  collectionId: number;
  onRepositoriesChanged: () => void;
  onClose: () => void;
}

export function ManageRepositoriesDialog({
  collectionId,
  onRepositoriesChanged,
  onClose,
}: ManageRepositoriesDialogProps) {
  const [activeTab, setActiveTab] = useState("starred");
  const [repos, setRepos] = useState<GithubRepo[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const { profile, dispatch } = useProfile();
  const [selectedRepos, setSelectedRepos] = useState<GithubRepo[]>([]);

  useEffect(() => {
    setRepos(profile?.starred_repositories || []);
  }, [profile?.starred_repositories]);

  const saveRepositories = async () => {
    setIsLoading(true);
    const collections = profile?.collections || [];
    const collection = collections.find(
      (collection) => collection.collection_id === collectionId,
    ) || { repositories: [] };
    const currentCollectionRepos = collection.repositories.map(
      ({ repository_id }) => repository_id,
    );
    const reposToAdd = selectedRepos.filter(
      (repo) => !currentCollectionRepos.includes(repo.id),
    );
    const reposToRemove = currentCollectionRepos.filter(
      (repoId) => !selectedRepos.some((repo) => repo.id === repoId),
    );

    const promises = [
      ...reposToAdd.map((repo) =>
        fetchWrapper(`/api/collections/${collectionId}/repositories`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ repository_id: repo.id }),
        }),
      ),
      ...reposToRemove.map((repoId) =>
        fetchWrapper(
          `/api/collections/${collectionId}/repositories/${repoId}`,
          {
            method: "DELETE",
          },
        ),
      ),
    ];

    try {
      await Promise.all(promises);

      reposToAdd.forEach((repo) => {
        dispatch({
          type: "ADD_REPOSITORY_TO_COLLECTION",
          payload: {
            collectionId: collectionId,
            repoId: repo.id,
            repository: {
              repository_id: repo.id,
              owner: repo.owner,
              name: repo.name,
              created_at: new Date(),
              updated_at: new Date(),
              indexed_at: null,
            },
          },
        });
      });

      reposToRemove.forEach((repoId) => {
        dispatch({
          type: "REMOVE_REPOSITORY_FROM_COLLECTION",
          payload: {
            collectionId: collectionId,
            repoId: repoId,
          },
        });
      });

      onRepositoriesChanged();
      // onClose();
    } catch (error) {
      console.error("Failed to save repositories:", error);
      toast({
        title: "Error",
        description: "Failed to remove repository. Please try again.",
        variant: "destructive",
      });
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <Tabs value={activeTab} onValueChange={setActiveTab}>
      <TabsList>
        <TabsTrigger value="starred">Starred</TabsTrigger>
        <TabsTrigger value="organization">Organization</TabsTrigger>
        <TabsTrigger value="search">Search</TabsTrigger>
      </TabsList>
      <TabsContent value="starred">
        <ManageRepositoriesTable
          collectionId={collectionId}
          repositories={repos}
          onSelectionChange={setSelectedRepos}
        />
      </TabsContent>
      <TabsContent value="organization">
        <p>Organization repositories will be displayed here.</p>
        {/* Implement organization repositories view */}
      </TabsContent>
      <TabsContent value="search">
        <p>Repository search will be implemented here.</p>
        {/* Implement repository search view */}
      </TabsContent>
      <DialogFooter>
        <Button onClick={onClose} variant="outline">
          Cancel
        </Button>
        <Button onClick={saveRepositories} disabled={isLoading}>
          {isLoading ? "Saving..." : "Save"}
        </Button>
      </DialogFooter>
    </Tabs>
  );
}
