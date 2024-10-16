// components/CollectionsList.tsx
import { useState, useEffect } from "react";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import Link from "next/link";
import { fetchWrapper } from "@/lib/fetchWrapper";
import { useProfile } from "@/contexts/ProfileContext";
import { getRepositoryCountForCollection } from "@/lib/collection";

export interface Collection {
  collection_id: number;
  name: string;
  description: string | null;
  is_default: boolean;
  created_at: string;
  updated_at: string;
  repository_count: number;
}

interface CollectionsListProps {
  refreshTrigger: number;
  onCollectionsUpdated?: (collections: Collection[]) => void;
}

export function CollectionsList({
  refreshTrigger,
  onCollectionsUpdated,
}: CollectionsListProps) {
  const [collections, setCollections] = useState<Collection[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const { profileData } = useProfile();

  console.log("Profile Data in Collections List:", profileData);

  useEffect(() => {
    const fetchCollections = async () => {
      try {
        const response = await fetchWrapper("/api/collections", {
          credentials: "include",
        });
        if (!response.ok) {
          throw new Error("Failed to fetch collections");
        }
        const data = await response.json();
        setCollections(data);
        if (onCollectionsUpdated) onCollectionsUpdated(data);
      } catch (error) {
        setError("Error fetching collections");
        console.error("Error fetching collections:", error);
      } finally {
        setIsLoading(false);
      }
    };

    fetchCollections();
  }, [refreshTrigger, onCollectionsUpdated]);

  if (isLoading) return <div>Loading collections...</div>;
  if (error) return <div>{error}</div>;

  return (
    <div>
      <h2 className="text-2xl font-semibold mb-4">Your Collections</h2>
      {collections.map((collection) => (
        <Card key={collection.collection_id} className="mb-4">
          <CardHeader>
            <CardTitle>{collection.name}</CardTitle>
          </CardHeader>
          <CardContent>
            <p>{collection.description || "No description"}</p>
            <p className="mt-2 text-sm text-gray-500">
              {profileData &&
                getRepositoryCountForCollection(
                  collection.collection_id,
                  profileData.repo_collections,
                )}{" "}
              {collection.repository_count === 1
                ? "repository"
                : "repositories"}
            </p>
            <Button asChild className="mt-2">
              <Link href={`/collections/${collection.collection_id}`}>
                View Collection
              </Link>
            </Button>
          </CardContent>
        </Card>
      ))}
    </div>
  );
}
