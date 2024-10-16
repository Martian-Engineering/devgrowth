// components/CollectionsList.tsx
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import Link from "next/link";
import { Collection } from "@/contexts/ProfileContext";

interface CollectionsListProps {
  collections: Collection[];
}

export function CollectionsList({ collections }: CollectionsListProps) {
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
              {collection.repositories.length}{" "}
              {collection.repositories.length === 1
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
