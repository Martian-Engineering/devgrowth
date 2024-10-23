// components/Repository.tsx
import { useCallback } from "react";
import {
  ChevronDownIcon,
  // CircleIcon,
  StarIcon,
  BookmarkIcon,
  BarChartIcon,
  ExternalLinkIcon,
} from "@radix-ui/react-icons";
import {
  Card,
  CardDescription,
  CardHeader,
  CardTitle,
  CardContent,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Separator } from "@/components/ui/separator";
import { useProfile, Collection } from "@/contexts/ProfileContext";
import { Repository } from "@/lib/repository";
import Link from "next/link";

interface RepositoryCardProps {
  repo: Repository;
  collections: Collection[];
  onCollectionUpdate?: () => void;
}

export function RepositoryCard({
  repo,
  collections,
  onCollectionUpdate,
}: RepositoryCardProps) {
  const { profile, dispatch } = useProfile();

  const toggleRepoInCollection = useCallback(
    async (collectionId: number) => {
      const isInCollection =
        profile &&
        profile.repo_collections &&
        profile.repo_collections[repo.repository_id]?.includes(collectionId);

      // Optimistic update
      if (isInCollection) {
        dispatch({
          type: "REMOVE_REPOSITORY_FROM_COLLECTION",
          payload: { repoId: repo.repository_id, collectionId },
        });
      } else {
        dispatch({
          type: "ADD_REPOSITORY_TO_COLLECTION",
          payload: {
            collectionId,
            repoId: repo.repository_id,
            repository: {
              repository_id: repo.repository_id,
              owner: repo.owner,
              name: repo.name,
              stargazers_count: repo.stargazers_count,
              description: repo.description,
              created_at: repo.created_at,
              updated_at: repo.updated_at,
              indexed_at: repo.indexed_at,
            },
          },
        });
      }

      try {
        let method, url, body;
        if (isInCollection) {
          method = "DELETE";
          url = `/api/collections/${collectionId}/repositories/${repo.repository_id}`;
          body = null;
        } else {
          method = "POST";
          url = `/api/collections/${collectionId}/repositories`;
          body = JSON.stringify({ repository_id: repo.repository_id });
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
          if (onCollectionUpdate) onCollectionUpdate();
        }
      } catch (error) {
        console.error("Error updating repository in collection:", error);
      }
    },
    [repo, profile, dispatch, onCollectionUpdate],
  );

  const isInCollection = (collectionId: number) =>
    profile &&
    profile.repo_collections &&
    profile.repo_collections[repo.repository_id]?.includes(collectionId);

  return (
    <Card>
      <CardHeader className="grid grid-cols-[1fr_110px] items-start gap-4 space-y-0">
        <div className="space-y-1">
          <CardTitle>{`${repo.owner}/${repo.name}`}</CardTitle>
          <CardDescription>
            {repo.description || "No description available."}
          </CardDescription>
        </div>
        <div className="flex items-center space-x-1 rounded-md bg-secondary text-secondary-foreground">
          <Button variant="secondary" className="px-3 shadow-none">
            <BarChartIcon className="mr-2 h-4 w-4" />
            <Link href={`/repositories/${repo.owner}/${repo.name}`}>View</Link>
          </Button>
          <Separator orientation="vertical" className="h-[20px]" />
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="secondary" className="px-2 shadow-none">
                <ChevronDownIcon className="h-4 w-4 text-secondary-foreground" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent
              align="end"
              alignOffset={-5}
              className="w-[200px]"
              forceMount
            >
              <DropdownMenuItem>
                <BookmarkIcon className="mr-2 h-4 w-4" /> Save
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem>
                <ExternalLinkIcon className="mr-2 h-4 w-4" />
                <a
                  href={`https://github.com/${repo.owner}/${repo.name}`}
                  target="_blank"
                >
                  View on GitHub
                </a>
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuLabel>Add to Collection</DropdownMenuLabel>
              <DropdownMenuSeparator />
              {collections.map((collection) => (
                <DropdownMenuCheckboxItem
                  key={collection.collection_id}
                  checked={isInCollection(collection.collection_id)}
                  onCheckedChange={() =>
                    toggleRepoInCollection(collection.collection_id)
                  }
                >
                  {collection.name}
                </DropdownMenuCheckboxItem>
              ))}
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </CardHeader>
      <CardContent>
        <div className="flex space-x-4 text-sm text-muted-foreground">
          {/* <div className="flex items-center">
            <CircleIcon className="mr-1 h-3 w-3 fill-sky-400 text-sky-400" />
            {repo.language || "Unknown"}
          </div> */}
          <div className="flex items-center">
            <StarIcon className="mr-1 h-3 w-3" />
            {repo.stargazers_count
              ? repo.stargazers_count.toLocaleString("en-US")
              : 0}
          </div>
          <div>
            Updated {new Date(repo.updated_at || null).toLocaleDateString()}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
