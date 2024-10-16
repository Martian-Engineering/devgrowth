"use client";

// TODO: make adding repositories independently of collections work again
import { useState, useEffect } from "react";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { AddRepositoryForm } from "@/components/AddRepositoryForm";
import { Toaster } from "@/components/ui/toaster";
import { fetchWrapper } from "@/lib/fetchWrapper";

interface Repository {
  id: number;
  name: string;
  owner: string;
  indexed_at: string | null;
  created_at: string;
  updated_at: string;
}

export default function RepositoriesPage() {
  const [repositories, setRepositories] = useState<Repository[]>([]);
  const [showAddForm, setShowAddForm] = useState(false);

  const fetchRepositories = async () => {
    const response = await fetchWrapper("/api/repositories");
    if (!response.ok) throw new Error("Failed to fetch repositories");
    const data = await response.json();
    setRepositories(data);
  };

  useEffect(() => {
    fetchRepositories();
  }, []);

  return (
    <div className="container mx-auto p-4">
      <Card>
        <CardHeader>
          <CardTitle className="flex justify-between items-center">
            Repositories
            <Button onClick={() => setShowAddForm(!showAddForm)}>
              {showAddForm ? "Cancel" : "Add Repository"}
            </Button>
          </CardTitle>
        </CardHeader>
        <CardContent>
          {showAddForm && (
            <AddRepositoryForm
              onRepositoryAdded={() => {
                fetchRepositories();
                setShowAddForm(false);
              }}
            />
          )}
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Name</TableHead>
                <TableHead>Owner</TableHead>
                <TableHead>Created At</TableHead>
                <TableHead>Updated At</TableHead>
                <TableHead>Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {repositories.map((repo) => (
                <TableRow key={repo.id}>
                  <TableCell>{repo.name}</TableCell>
                  <TableCell>{repo.owner}</TableCell>
                  <TableCell>
                    {new Date(repo.created_at).toLocaleDateString()}
                  </TableCell>
                  <TableCell>
                    {new Date(repo.updated_at).toLocaleDateString()}
                  </TableCell>
                  <TableCell>
                    <Button asChild>
                      <Link href={`/repository/${repo.owner}/${repo.name}`}>
                        View
                      </Link>
                    </Button>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
      <Toaster />
    </div>
  );
}
