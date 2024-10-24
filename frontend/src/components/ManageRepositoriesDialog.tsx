// devgrowth/frontend/src/components/ManageRepositoriesDialog.tsx
import React, { useState, useEffect } from "react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ManageRepositoriesTable } from "@/components/ManageRepositoriesTable";
import { Button } from "@/components/ui/button";
import { DialogFooter } from "@/components/ui/dialog";
import { fetchWrapper } from "@/lib/fetchWrapper";
import { useProfile } from "@/contexts/ProfileContext";
import { Repository, GithubRepo, parseGithubRepos } from "@/lib/repository";

import { ReloadIcon } from "@radix-ui/react-icons";

import { toast } from "@/hooks/use-toast";
import { PaginatedResponse } from "@/types/PaginatedResponse";

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
  const [starredRepos, setStarredRepos] = useState<Repository[]>([]);
  const [orgRepos, setOrgRepos] = useState<Repository[]>([]);
  const [searchRepos, setSearchRepos] = useState<Repository[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const { profile, dispatch } = useProfile();
  const [selectedRepos, setSelectedRepos] = useState<Repository[]>([]);
  const [orgName, setOrgName] = useState("");
  const [isFetchingOrg, setIsFetchingOrg] = useState(false);
  const [searchTerm, setSearchTerm] = useState("");
  const [isSearching, setIsSearching] = useState(false);
  const [orgPage, setOrgPage] = useState(1);
  const [orgTotalPages, setOrgTotalPages] = useState(1);
  const pageSize = 10;

  useEffect(() => {
    if (activeTab === "starred") {
      setStarredRepos(profile?.starred_repositories || []);
    }
  }, [activeTab, profile?.starred_repositories]);

  useEffect(() => {
    const collections = profile?.collections || [];
    const collection = collections.find(
      (collection) => collection.collection_id === collectionId,
    ) || { repositories: [] };
    setSelectedRepos(
      collection.repositories.map((repo) => {
        return {
          repository_id: repo.repository_id,
          owner: repo.owner,
          name: repo.name,
          description: repo.description,
          stargazers_count: repo.stargazers_count,
          indexed_at: repo.indexed_at,
          created_at: repo.created_at,
          updated_at: repo.updated_at,
        };
      }),
    );
  }, [collectionId, profile?.collections]);

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
      (repo) => !currentCollectionRepos.includes(repo.repository_id),
    );
    const reposToRemove = currentCollectionRepos.filter(
      (repoId) => !selectedRepos.some((repo) => repo.repository_id === repoId),
    );

    const promises = [
      ...reposToAdd.map((repo) =>
        fetchWrapper(`/api/collections/${collectionId}/repositories`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ repository_id: repo.repository_id }),
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
            repoId: repo.repository_id,
            repository: {
              repository_id: repo.repository_id,
              owner: repo.owner,
              name: repo.name,
              description: repo.description,
              stargazers_count: repo.stargazers_count,
              created_at: new Date().toISOString(),
              updated_at: new Date().toISOString(),
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

  const fetchOrgRepositories = async (pageNumber: number) => {
    if (!orgName) return;
    setIsFetchingOrg(true);
    try {
      const response = await fetchWrapper(
        `/api/github/orgs/${orgName}/repos?page=${pageNumber}&page_size=${pageSize}`,
      );
      if (response.ok) {
        const data: PaginatedResponse<GithubRepo> = await response.json();
        setOrgRepos(parseGithubRepos(data.data));
        setOrgTotalPages(data.total_pages);
        setOrgPage(data.page);
        console.log(data);
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

  const handleOrgPageChange = (newPage: number) => {
    setOrgPage(newPage);
    fetchOrgRepositories(newPage);
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
        setSearchRepos(parseGithubRepos(data));
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
        <form
          className="space-y-4"
          onSubmit={(e) => {
            e.preventDefault();
            fetchOrgRepositories(1);
          }}
        >
          <div>
            <Label htmlFor="orgName">Organization Name</Label>
            <Input
              id="orgName"
              value={orgName}
              onChange={(e) => setOrgName(e.target.value)}
              placeholder="Enter organization name"
            />
          </div>
          <Button type="submit" disabled={isFetchingOrg}>
            {isFetchingOrg ? (
              <>
                <ReloadIcon className="mr-2 h-4 w-4 animate-spin" />
                Fetching...
              </>
            ) : (
              "Fetch Repositories"
            )}
          </Button>
        </form>
        {orgRepos.length > 0 && (
          <ManageRepositoriesTable
            collectionId={collectionId}
            repositories={orgRepos}
            onSelectionChange={setSelectedRepos}
            totalPages={orgTotalPages}
            currentPage={orgPage}
            onPageChange={handleOrgPageChange}
            isLoading={isFetchingOrg}
            serverSidePagination={true}
          />
        )}
      </TabsContent>
      <TabsContent value="search">
        <form
          className="space-y-4"
          onSubmit={(e) => {
            e.preventDefault();
            searchRepositories();
          }}
        >
          <div>
            <Label htmlFor="searchTerm">Search Repositories</Label>
            <Input
              id="searchTerm"
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              placeholder="Enter search term"
            />
          </div>
          <Button type="submit" disabled={isSearching}>
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
        </form>
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
