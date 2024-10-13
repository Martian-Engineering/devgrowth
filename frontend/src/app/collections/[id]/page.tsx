// src/app/collections/[id]/page.tsx
"use client";

import { useState, useEffect } from "react";
import { useParams } from "next/navigation";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogTrigger,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { AddRepositoryForm } from "@/components/AddRepositoryForm";
import { toast } from "@/hooks/use-toast";

interface Collection {
  collection_id: number;
  name: string;
  description: string;
  repositories: Repository[];
}

interface Repository {
  repository_id: number;
  name: string;
  owner: string;
  indexed_at: Date | null;
  created_at: Date;
  updated_at: Date;
}

export default function CollectionPage() {
  const { id } = useParams();
  const [collection, setCollection] = useState<Collection | null>(null);
  const [isDialogOpen, setIsDialogOpen] = useState(false);

  useEffect(() => {
    fetchCollection();
  }, [id]);

  const fetchCollection = async () => {
    try {
      const response = await fetch(`/api/collections/${id}`);
      if (!response.ok) throw new Error("Failed to fetch collection");
      const data = await response.json();
      console.log("Collection data:", data);
      setCollection(data);
    } catch (error) {
      console.error("Error fetching collection:", error);
      toast({
        title: "Error",
        description: "Failed to fetch collection. Please try again.",
        variant: "destructive",
      });
    }
  };

  const handleRepositoryAdded = () => {
    setIsDialogOpen(false);
    fetchCollection();
  };

  const handleRemoveRepository = async (repoId: number) => {
    try {
      const response = await fetch(
        `/api/collections/${id}/repositories/${repoId}`,
        {
          method: "DELETE",
        },
      );
      if (!response.ok) throw new Error("Failed to remove repository");
      fetchCollection();
      toast({
        title: "Repository removed",
        description: "Successfully removed repository from collection",
      });
    } catch (error) {
      console.error("Error removing repository:", error);
      toast({
        title: "Error",
        description: "Failed to remove repository. Please try again.",
        variant: "destructive",
      });
    }
  };

  if (!collection) return <div>Loading...</div>;

  return (
    <div className="container mx-auto p-4">
      <h1 className="text-2xl font-bold mb-4">{collection.name}</h1>
      <p className="mb-4">{collection.description}</p>

      <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
        <DialogTrigger asChild>
          <Button>Add Repository</Button>
        </DialogTrigger>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Add Repository to Collection</DialogTitle>
          </DialogHeader>
          <AddRepositoryForm
            onRepositoryAdded={handleRepositoryAdded}
            collectionId={collection.collection_id}
          />
        </DialogContent>
      </Dialog>

      <div className="mt-8">
        <h2 className="text-xl font-semibold mb-4">Repositories</h2>
        {collection.repositories.map((repo) => (
          <div key={repo.repository_id} className="border p-4 mb-4 rounded">
            <h3 className="font-bold">
              {repo.owner}/{repo.name}
            </h3>
            <a
              href={`https://github.com/${repo.owner}/${repo.name}`}
              target="_blank"
              rel="noopener noreferrer"
              className="text-blue-500 hover:underline"
            >
              View on GitHub
            </a>
            <Button
              variant="destructive"
              size="sm"
              className="ml-4"
              onClick={() => handleRemoveRepository(repo.repository_id)}
            >
              Remove
            </Button>
          </div>
        ))}
      </div>
    </div>
  );
}
