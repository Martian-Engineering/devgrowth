// components/StarredReposList.tsx
import { useState, useEffect, useCallback } from "react";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuCheckboxItem,
} from "@/components/ui/dropdown-menu";
import { PlusIcon } from "@radix-ui/react-icons";
import {
  Pagination,
  PaginationContent,
  PaginationItem,
  PaginationLink,
  PaginationNext,
  PaginationPrevious,
} from "@/components/ui/pagination";

const ITEMS_PER_PAGE = 10;

interface StarredRepo {
  id: number;
  name: string;
  owner: string;
  html_url: string;
  description: string | null;
  stargazers_count: number | null;
}

interface Collection {
  collection_id: number;
  name: string;
}

interface RepoCollectionMap {
  [repoId: number]: number[];
}

interface StarredReposListProps {
  repos: StarredRepo[];
  repoCollections: RepoCollectionMap;
  onCollectionUpdate: () => void;
}

export function StarredReposList({
  repos,
  repoCollections,
  onCollectionUpdate,
}: StarredReposListProps) {
  const [collections, setCollections] = useState<Collection[]>([]);
  const [currentPage, setCurrentPage] = useState(1);
  const [localRepoCollections, setLocalRepoCollections] =
    useState<RepoCollectionMap>(repoCollections);

  useEffect(() => {
    setLocalRepoCollections(repoCollections);
  }, [repoCollections]);

  const fetchCollections = useCallback(async () => {
    try {
      const response = await fetch("/api/collections", {
        credentials: "include",
      });
      if (!response.ok) throw new Error("Failed to fetch collections");
      const data = await response.json();
      setCollections(data);
    } catch (error) {
      console.error("Error fetching collections:", error);
    }
  }, []);

  useEffect(() => {
    fetchCollections();
  }, [fetchCollections]);

  const toggleRepoInCollection = async (
    repoId: number,
    collectionId: number,
  ) => {
    const isInCollection = localRepoCollections[repoId]?.includes(collectionId);
    // Optimistic update
    setLocalRepoCollections((prev) => {
      const newCollections = { ...prev };
      if (isInCollection) {
        newCollections[repoId] = newCollections[repoId].filter(
          (id) => id !== collectionId,
        );
      } else {
        newCollections[repoId] = [
          ...(newCollections[repoId] || []),
          collectionId,
        ];
      }
      return newCollections;
    });

    try {
      let method, url, body;
      if (isInCollection) {
        method = "DELETE";
        url = `/api/collections/${collectionId}/repositories/${repoId}`;
        body = null;
      } else {
        method = "POST";
        url = `/api/collections/${collectionId}/repositories`;
        body = JSON.stringify({ repository_id: repoId });
      }
      const response = await fetch(url, {
        method,
        body,
      });

      if (!response.ok) {
        console.error(response);
        throw new Error(
          `Failed to ${isInCollection ? "remove from" : "add to"} collection`,
        );
      } else {
        onCollectionUpdate();
      }
    } catch (error) {
      console.error("Error updating repository in collection:", error);
      setLocalRepoCollections(repoCollections);
    }
  };

  const totalPages = Math.ceil(repos.length / ITEMS_PER_PAGE);
  const paginatedRepos = repos.slice(
    (currentPage - 1) * ITEMS_PER_PAGE,
    currentPage * ITEMS_PER_PAGE,
  );

  return (
    <div>
      <h2 className="text-2xl font-semibold mb-4">Your Starred Repositories</h2>
      {paginatedRepos.map((repo) => (
        <Card key={repo.id} className="mb-4">
          <CardHeader>
            <CardTitle className="flex justify-between items-center">
              {`${repo.owner}/${repo.name}`}
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button variant="outline" size="icon">
                    <PlusIcon className="h-4 w-4" />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent>
                  {collections.map((collection) => (
                    <DropdownMenuCheckboxItem
                      key={collection.collection_id}
                      checked={localRepoCollections[repo.id]?.includes(
                        collection.collection_id,
                      )}
                      onCheckedChange={() =>
                        toggleRepoInCollection(
                          repo.id,
                          collection.collection_id,
                        )
                      }
                    >
                      {collection.name}
                    </DropdownMenuCheckboxItem>
                  ))}
                </DropdownMenuContent>
              </DropdownMenu>
            </CardTitle>
          </CardHeader>
          <CardContent>
            {repo.description && <p>{repo.description}</p>}
            <p>Stars: {repo.stargazers_count ?? "N/A"}</p>
            <Button asChild className="mt-2">
              <a href={repo.html_url} target="_blank" rel="noopener noreferrer">
                View on GitHub
              </a>
            </Button>
          </CardContent>
        </Card>
      ))}

      {totalPages > 1 && (
        <Pagination>
          <PaginationContent>
            {currentPage > 1 ? (
              <PaginationItem>
                <PaginationPrevious
                  onClick={() =>
                    setCurrentPage((prev) => Math.max(prev - 1, 1))
                  }
                />
              </PaginationItem>
            ) : (
              <PaginationItem>
                <PaginationPrevious className="pointer-events-none opacity-50" />
              </PaginationItem>
            )}
            {[...Array(totalPages)].map((_, index) => (
              <PaginationItem key={index}>
                <PaginationLink
                  onClick={() => setCurrentPage(index + 1)}
                  isActive={currentPage === index + 1}
                >
                  {index + 1}
                </PaginationLink>
              </PaginationItem>
            ))}
            {currentPage < totalPages ? (
              <PaginationItem>
                <PaginationNext
                  onClick={() =>
                    setCurrentPage((prev) => Math.min(prev + 1, totalPages))
                  }
                />
              </PaginationItem>
            ) : (
              <PaginationItem>
                <PaginationNext className="pointer-events-none opacity-50" />
              </PaginationItem>
            )}
          </PaginationContent>
        </Pagination>
      )}
    </div>
  );
}
