// components/CollectionsList.tsx
import { useState, useEffect } from "react";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import Link from "next/link";

interface Collection {
  collection_id: number;
  name: string;
  description: string | null;
  is_default: boolean;
}

export function CollectionsList() {
  const [collections, setCollections] = useState<Collection[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchCollections = async () => {
      try {
        const response = await fetch("/api/collections", {
          credentials: "include",
        });
        if (!response.ok) {
          throw new Error("Failed to fetch collections");
        }
        const data = await response.json();
        setCollections(data);
      } catch (error) {
        setError("Error fetching collections");
        console.error("Error fetching collections:", error);
      } finally {
        setIsLoading(false);
      }
    };

    fetchCollections();
  }, []);

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
