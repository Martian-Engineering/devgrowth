// components/GithubRepoCard.tsx
import { useCallback } from "react";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuCheckboxItem,
} from "@/components/ui/dropdown-menu";
import { PlusIcon } from "@radix-ui/react-icons";
import { useProfile, GithubRepo, Collection } from "@/contexts/ProfileContext";

interface GithubRepoCardProps {
  repo: GithubRepo;
  collections: Collection[];
  onCollectionUpdate: () => void;
}

export function GithubRepoCard({
  repo,
  collections,
  onCollectionUpdate,
}: GithubRepoCardProps) {
  const { profile, dispatch } = useProfile();

  const toggleRepoInCollection = useCallback(
    async (collectionId: number) => {
      const isInCollection =
        profile &&
        profile.repo_collections &&
        profile.repo_collections[repo.id]?.includes(collectionId);

      // Optimistic update
      if (isInCollection) {
        dispatch({
          type: "REMOVE_REPOSITORY_FROM_COLLECTION",
          payload: { repoId: repo.id, collectionId },
        });
      } else {
        dispatch({
          type: "ADD_REPOSITORY_TO_COLLECTION",
          payload: {
            collectionId,
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
      }

      try {
        let method, url, body;
        if (isInCollection) {
          method = "DELETE";
          url = `/api/collections/${collectionId}/repositories/${repo.id}`;
          body = null;
        } else {
          method = "POST";
          url = `/api/collections/${collectionId}/repositories`;
          body = JSON.stringify({ repository_id: repo.id });
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
      }
    },
    [repo, profile, dispatch, onCollectionUpdate],
  );

  return (
    <Card className="mb-4">
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
                  checked={
                    !!(
                      profile &&
                      profile.repo_collections &&
                      profile.repo_collections[repo.id]?.includes(
                        collection.collection_id,
                      )
                    )
                  }
                  onCheckedChange={() =>
                    toggleRepoInCollection(collection.collection_id)
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
  );
}
