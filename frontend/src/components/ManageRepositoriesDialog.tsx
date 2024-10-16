// devgrowth/frontend/src/components/ManageRepositoriesDialog.tsx
import React, { useState, useEffect } from "react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ManageRepositoriesTable } from "@/components/ManageRepositoriesTable";
import { Button } from "@/components/ui/button";
import { DialogFooter } from "@/components/ui/dialog";
import { fetchWrapper } from "@/lib/fetchWrapper";
import { GithubRepo, useProfile } from "@/contexts/ProfileContext";

import { ReloadIcon } from "@radix-ui/react-icons";

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
  const [starredRepos, setStarredRepos] = useState<GithubRepo[]>([]);
  const [orgRepos, setOrgRepos] = useState<GithubRepo[]>([]);
  const [searchRepos, setSearchRepos] = useState<GithubRepo[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const { profile, dispatch } = useProfile();
  const [selectedRepos, setSelectedRepos] = useState<GithubRepo[]>([]);
  const [orgName, setOrgName] = useState("");
  const [isFetchingOrg, setIsFetchingOrg] = useState(false);
  const [searchTerm, setSearchTerm] = useState("");
  const [isSearching, setIsSearching] = useState(false);

  useEffect(() => {
    if (activeTab === "starred") {
      setStarredRepos(profile?.starred_repositories || []);
    }
  }, [activeTab, profile?.starred_repositories]);

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

  const fetchOrgRepositories = async () => {
    if (!orgName) return;
    setIsFetchingOrg(true);
    try {
      const response = await fetchWrapper(`/api/github/orgs/${orgName}/repos`);
      if (response.ok) {
        const data = await response.json();
        setOrgRepos(data);
      } else {
        toast({
          title: "Error",
          description: "Failed to fetch organization repositories.",
          variant: "destructive",
        });
      }
    } catch (error) {
      console.error("Error fetching organization repositories:", error);
      toast({
        title: "Error",
        description: `An error occurred while fetching ${orgName}'s repositories.`,
        variant: "destructive",
      });
    } finally {
      setIsFetchingOrg(false);
    }
  };

  const searchRepositories = async () => {
    if (!searchTerm) return;
    setIsSearching(true);
    try {
      const response = await fetchWrapper(
        `/api/github/search?q=${encodeURIComponent(searchTerm)}`,
      );
      if (response.ok) {
        const data = await response.json();
        setSearchRepos(data);
      } else {
        toast({
          title: "Error",
          description: "Failed to search repositories.",
          variant: "destructive",
        });
      }
    } catch (error) {
      console.error("Error searching repositories:", error);
      toast({
        title: "Error",
        description: "An error occurred while searching repositories.",
        variant: "destructive",
      });
    } finally {
      setIsSearching(false);
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
          repositories={starredRepos}
          onSelectionChange={setSelectedRepos}
        />
      </TabsContent>
      <TabsContent value="organization">
        <div className="space-y-4">
          <div>
            <Label htmlFor="orgName">Organization Name</Label>
            <Input
              id="orgName"
              value={orgName}
              onChange={(e) => setOrgName(e.target.value)}
              placeholder="Enter organization name"
            />
          </div>
          <Button onClick={fetchOrgRepositories} disabled={isFetchingOrg}>
            {isFetchingOrg ? (
              <>
                <ReloadIcon className="mr-2 h-4 w-4 animate-spin" />
                Fetching...
              </>
            ) : (
              "Fetch Repositories"
            )}
          </Button>
          {orgRepos.length > 0 && (
            <ManageRepositoriesTable
              collectionId={collectionId}
              repositories={orgRepos}
              onSelectionChange={setSelectedRepos}
            />
          )}
        </div>
      </TabsContent>
      <TabsContent value="search">
        <div className="space-y-4">
          <div>
            <Label htmlFor="searchTerm">Search Repositories</Label>
            <Input
              id="searchTerm"
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              placeholder="Enter search term"
            />
          </div>
          <Button onClick={searchRepositories} disabled={isSearching}>
            {isSearching ? (
              <>
                <ReloadIcon className="mr-2 h-4 w-4 animate-spin" />
                Searching...
              </>
            ) : (
              "Search"
            )}
          </Button>
          {searchRepos.length > 0 && (
            <ManageRepositoriesTable
              collectionId={collectionId}
              repositories={searchRepos}
              onSelectionChange={setSelectedRepos}
            />
          )}
        </div>
      </TabsContent>
      <DialogFooter>
        <Button onClick={onClose} variant="outline" disabled={isLoading}>
          Cancel
        </Button>
        <Button onClick={saveRepositories} disabled={isLoading}>
          {isLoading ? (
            <>
              <ReloadIcon className="mr-2 h-4 w-4 animate-spin" />
              Saving...
            </>
          ) : (
            "Save"
          )}
        </Button>
      </DialogFooter>
    </Tabs>
  );
}
