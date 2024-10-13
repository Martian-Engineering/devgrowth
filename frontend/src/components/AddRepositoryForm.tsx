// src/components/AddRepositoryForm.tsx
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { toast } from "@/hooks/use-toast";

interface AddRepositoryFormProps {
  onRepositoryAdded: () => void;
  collectionId: number;
}

export function AddRepositoryForm({
  onRepositoryAdded,
  collectionId,
}: AddRepositoryFormProps) {
  const [owner, setOwner] = useState("");
  const [name, setName] = useState("");

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      const response = await fetch(
        `/api/collections/${collectionId}/repositories`,
        {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({ owner, name }),
        },
      );

      if (!response.ok) {
        throw new Error("Failed to add repository");
      }

      toast({
        title: "Repository added",
        description: `Successfully added ${owner}/${name} to the collection`,
      });

      setOwner("");
      setName("");
      onRepositoryAdded();
    } catch (error) {
      console.error("Error adding repository:", error);
      toast({
        title: "Error",
        description: "Failed to add repository. Please try again.",
        variant: "destructive",
      });
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div>
        <Label htmlFor="owner">Repository Owner</Label>
        <Input
          id="owner"
          value={owner}
          onChange={(e) => setOwner(e.target.value)}
          placeholder="e.g., octocat"
          required
        />
      </div>
      <div>
        <Label htmlFor="name">Repository Name</Label>
        <Input
          id="name"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="e.g., Hello-World"
          required
        />
      </div>
      <Button type="submit">Add Repository</Button>
    </form>
  );
}
