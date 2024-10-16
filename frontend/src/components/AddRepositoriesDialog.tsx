// devgrowth/frontend/src/components/AddRepositoriesDialog.tsx
import React, { useState, useEffect } from "react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  AddRepositoriesForm,
  Repository,
} from "@/components/AddRepositoriesForm";
import { Button } from "@/components/ui/button";
import { DialogFooter } from "@/components/ui/dialog";
import { fetchWrapper } from "@/lib/fetchWrapper";

interface AddRepositoriesDialogProps {
  collectionId: number;
  onRepositoriesAdded: () => void;
  onClose: () => void;
}

export function AddRepositoriesDialog({
  collectionId,
  onRepositoriesAdded,
  onClose,
}: AddRepositoriesDialogProps) {
  const [activeTab, setActiveTab] = useState("starred");
  const [starredRepos, setStarredRepos] = useState<Repository[]>([]);
  const [rowSelection, setRowSelection] = useState<Record<string, boolean>>({});
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    if (activeTab === "starred") {
      fetchStarredRepos();
    }
  }, [activeTab]);

  const fetchStarredRepos = async () => {
    try {
      const response = await fetchWrapper("/api/github/starred");
      if (!response.ok) throw new Error("Failed to fetch starred repositories");
      const data = await response.json();
      setStarredRepos(data);
    } catch (error) {
      console.error("Error fetching starred repositories:", error);
      // You can add a toast notification here
    }
  };

  const addRepositories = async () => {
    setIsLoading(true);
    const selectedRepos = Object.keys(rowSelection)
      .filter((key) => rowSelection[key])
      .map((key) => starredRepos[parseInt(key)]);

    for (const repo of selectedRepos) {
      try {
        const response = await fetchWrapper(
          `/api/collections/${collectionId}/repositories`,
          {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
            },
            body: JSON.stringify({
              owner: repo.owner,
              name: repo.name,
            }),
          },
        );

        if (!response.ok) {
          throw new Error(
            `Failed to add repository ${repo.owner}/${repo.name}`,
          );
        }
      } catch (error) {
        console.error(error);
        // Optionally, you can handle errors here (e.g., show a toast notification)
      }
    }

    setIsLoading(false);
    onRepositoriesAdded();
    onClose();
  };

  return (
    <Tabs value={activeTab} onValueChange={setActiveTab}>
      <TabsList>
        <TabsTrigger value="starred">Starred</TabsTrigger>
        <TabsTrigger value="organization">Organization</TabsTrigger>
        <TabsTrigger value="search">Search</TabsTrigger>
      </TabsList>
      <TabsContent value="starred">
        <AddRepositoriesForm
          repositories={starredRepos}
          rowSelection={rowSelection}
          setRowSelection={setRowSelection}
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
        <Button onClick={addRepositories} disabled={isLoading}>
          {isLoading ? "Adding..." : "Add Selected Repositories"}
        </Button>
      </DialogFooter>
    </Tabs>
  );
}
