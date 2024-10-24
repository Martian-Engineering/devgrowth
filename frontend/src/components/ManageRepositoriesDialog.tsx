// devgrowth/frontend/src/components/ManageRepositoriesDialog.tsx
import React, { useState, useEffect } from "react";
import {
  useReactTable,
  getCoreRowModel,
  getPaginationRowModel,
  ColumnDef,
  flexRender,
} from "@tanstack/react-table";
import {
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  TableCell,
} from "@/components/ui/table";
import {
  Pagination,
  PaginationContent,
  PaginationEllipsis,
  PaginationItem,
  PaginationLink,
  PaginationNext,
  PaginationPrevious,
} from "@/components/ui/pagination";
import { Checkbox } from "@/components/ui/checkbox";
import {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuItem,
} from "@/components/ui/dropdown-menu";
import { MoreHorizontal } from "lucide-react";
import {
  ChevronLeftIcon,
  ChevronRightIcon,
  ReloadIcon,
} from "@radix-ui/react-icons";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { DialogFooter } from "@/components/ui/dialog";
import { fetchWrapper } from "@/lib/fetchWrapper";
import { useProfile } from "@/contexts/ProfileContext";
import { Repository, GithubRepo, parseGithubRepos } from "@/lib/repository";
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
  const [isLoading, setIsLoading] = useState(false);
  const { profile, dispatch } = useProfile();
  const [selectedRepos, setSelectedRepos] = useState<Repository[]>([]);

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

  return (
    <Tabs value={activeTab} onValueChange={setActiveTab}>
      <TabsList>
        <TabsTrigger value="starred">Starred</TabsTrigger>
        <TabsTrigger value="organization">Organization</TabsTrigger>
        <TabsTrigger value="search">Search</TabsTrigger>
      </TabsList>
      <StarredTab
        collectionId={collectionId}
        onSelectionChange={setSelectedRepos}
      />
      <OrganizationsTab
        collectionId={collectionId}
        onSelectionChange={setSelectedRepos}
      />
      <SearchTab
        collectionId={collectionId}
        onSelectionChange={setSelectedRepos}
      />
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

function StarredTab({
  collectionId,
  onSelectionChange,
}: {
  collectionId: number;
  onSelectionChange: (repos: Repository[]) => void;
}) {
  const [repositories, setRepositories] = useState<Repository[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [page, setPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);
  const pageSize = 10;

  const fetchRepositories = async (pageNumber: number) => {
    setIsLoading(true);
    try {
      const response = await fetchWrapper(
        `/api/github/starred?page=${pageNumber}&page_size=${pageSize}`,
      );
      const data: PaginatedResponse<GithubRepo> = await response.json();
      setRepositories(parseGithubRepos(data.data));
      setTotalPages(data.total_pages);
      setPage(data.page);
    } catch (error) {
      console.error("Failed to fetch starred repositories:", error);
      toast({
        title: "Error",
        description: "Failed to fetch starred repositories.",
        variant: "destructive",
      });
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchRepositories(1);
  }, []);

  return (
    <TabsContent value="starred">
      {repositories.length > 0 && (
        <ManageRepositoriesTable
          collectionId={collectionId}
          repositories={repositories}
          onSelectionChange={onSelectionChange}
          totalPages={totalPages}
          currentPage={page}
          onPageChange={(newPage) => fetchRepositories(newPage)}
          isLoading={isLoading}
          serverSidePagination={true}
        />
      )}
    </TabsContent>
  );
}

function OrganizationsTab({
  collectionId,
  onSelectionChange,
}: {
  collectionId: number;
  onSelectionChange: (repos: Repository[]) => void;
}) {
  const [repositories, setRepositories] = useState<Repository[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [orgName, setOrgName] = useState("");
  const [page, setPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);
  const pageSize = 10;

  const fetchRepositories = async (pageNumber: number) => {
    if (!orgName) return;
    setIsLoading(true);
    try {
      const response = await fetchWrapper(
        `/api/github/orgs/${orgName}/repos?page=${pageNumber}&page_size=${pageSize}`,
      );
      const data: PaginatedResponse<GithubRepo> = await response.json();
      setRepositories(parseGithubRepos(data.data));
      setTotalPages(data.total_pages);
      setPage(data.page);
    } catch (error) {
      console.error("Error fetching organization repositories:", error);
      toast({
        title: "Error",
        description: "Failed to fetch starred repositories.",
        variant: "destructive",
      });
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <TabsContent value="organization">
      <form
        className="space-y-4"
        onSubmit={(e) => {
          e.preventDefault();
          fetchRepositories(1);
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
        <Button type="submit" disabled={isLoading}>
          {isLoading ? (
            <>
              <ReloadIcon className="mr-2 h-4 w-4 animate-spin" />
              Fetching...
            </>
          ) : (
            "Fetch Repositories"
          )}
        </Button>
      </form>
      {repositories.length > 0 && (
        <ManageRepositoriesTable
          collectionId={collectionId}
          repositories={repositories}
          onSelectionChange={onSelectionChange}
          totalPages={totalPages}
          currentPage={page}
          onPageChange={(newPage) => fetchRepositories(newPage)}
          isLoading={isLoading}
          serverSidePagination={true}
        />
      )}
    </TabsContent>
  );
}

function SearchTab({
  collectionId,
  onSelectionChange,
}: {
  collectionId: number;
  onSelectionChange: (repos: Repository[]) => void;
}) {
  const [repositories, setRepositories] = useState<Repository[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [searchTerm, setSearchTerm] = useState("");
  const [page, setPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);
  const pageSize = 10;

  const searchRepositories = async (pageNumber: number) => {
    if (!searchTerm) return;
    setIsLoading(true);
    try {
      const response = await fetchWrapper(
        `/api/github/search?q=${encodeURIComponent(searchTerm)}&page=${pageNumber}&page_size=${pageSize}`,
      );
      if (response.ok) {
        const data = await response.json();
        console.log(response);
        setRepositories(parseGithubRepos(data.data));
        setTotalPages(data.total_pages);
        setPage(data.page);
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
      setIsLoading(false);
    }
  };

  return (
    <TabsContent value="search">
      <form
        className="space-y-4"
        onSubmit={(e) => {
          e.preventDefault();
          searchRepositories(1);
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
        <Button type="submit" disabled={isLoading}>
          {isLoading ? (
            <>
              <ReloadIcon className="mr-2 h-4 w-4 animate-spin" />
              Searching...
            </>
          ) : (
            "Search"
          )}
        </Button>
        {repositories.length > 0 && (
          <ManageRepositoriesTable
            collectionId={collectionId}
            repositories={repositories}
            onSelectionChange={onSelectionChange}
            totalPages={totalPages}
            currentPage={page}
            onPageChange={(newPage) => searchRepositories(newPage)}
            isLoading={isLoading}
            serverSidePagination={true}
          />
        )}
      </form>
    </TabsContent>
  );
}

interface ManageRepositoriesTableProps {
  collectionId: number;
  repositories: Repository[];
  onSelectionChange: (selectedRepos: Repository[]) => void;
  totalPages?: number;
  currentPage?: number;
  onPageChange?: (newPage: number) => void;
  isLoading?: boolean;
  serverSidePagination?: boolean;
}

function ManageRepositoriesTable({
  collectionId,
  repositories,
  onSelectionChange,
  totalPages,
  currentPage,
  onPageChange,
  isLoading,
  serverSidePagination = false,
}: ManageRepositoriesTableProps) {
  const { profile } = useProfile();
  const [selection, setSelection] = useState<Set<number>>(new Set());
  const [page, setPage] = useState(1);

  useEffect(() => {
    const collections = profile?.collections || [];
    const collection = collections.find(
      (collection) => collection.collection_id === collectionId,
    ) || { repositories: [] };
    const currentCollectionRepos = collection.repositories.map(
      ({ repository_id }) => repository_id,
    );
    setSelection(new Set(currentCollectionRepos));
  }, [profile?.collections, collectionId]);

  const toggleSelection = (repoId: number) => {
    const newSelection = new Set(selection);
    if (newSelection.has(repoId)) {
      newSelection.delete(repoId);
    } else {
      newSelection.add(repoId);
    }
    setSelection(newSelection);

    const collections = profile?.collections || [];
    const collection = collections.find(
      (collection) => collection.collection_id === collectionId,
    ) || { repositories: [] };
    const filterRepositories = repositories.concat(collection.repositories);
    onSelectionChange(
      filterRepositories.filter((repo) => newSelection.has(repo.repository_id)),
    );
  };

  const columns: ColumnDef<Repository>[] = [
    {
      id: "select",
      header: () => (
        <Checkbox
          checked={
            repositories.length > 0 && selection.size === repositories.length
          }
          onCheckedChange={(value) => {
            const newSelection: Set<number> = value
              ? new Set(repositories.map((repo) => repo.repository_id))
              : new Set();
            setSelection(newSelection);
            onSelectionChange(
              repositories.filter((repo) =>
                newSelection.has(repo.repository_id),
              ),
            );
          }}
          aria-label="Select all"
        />
      ),
      cell: ({ row }) => {
        const repo = row.original;
        return (
          <Checkbox
            checked={selection.has(repo.repository_id)}
            onCheckedChange={() => {
              console.log("toggling selection", repo);
              toggleSelection(repo.repository_id);
            }}
            aria-label="Select row"
          />
        );
      },
    },
    {
      header: "Repository",
      accessorFn: (row: Repository) => `${row.owner}/${row.name}`,
    },
    {
      header: "Description",
      accessorKey: "description",
      cell: ({ getValue }) => (
        <div className="truncate max-w-xs">{getValue() as string}</div>
      ),
    },
    {
      header: "Stars",
      accessorKey: "stargazers_count",
    },
    {
      id: "actions",
      cell: ({ row }) => (
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" className="h-8 w-8 p-0">
              <MoreHorizontal className="h-4 w-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem
              onClick={() => {
                const { name, owner } = row.original;
                const url = `https://github.com/${owner}/${name}`;
                window.open(url, "_blank");
              }}
            >
              View on GitHub
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      ),
    },
  ];

  const table = useReactTable({
    data: repositories,
    columns,
    getCoreRowModel: getCoreRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
  });

  const handlePageChange = (newPage: number) => {
    setPage(newPage);
    table.setPageIndex(newPage - 1);
  };

  return (
    <div>
      <Table>
        <TableHeader>
          {table.getHeaderGroups().map((headerGroup) => (
            <TableRow key={headerGroup.id}>
              {headerGroup.headers.map((header) => (
                <TableHead key={header.id}>
                  {header.isPlaceholder
                    ? null
                    : flexRender(
                        header.column.columnDef.header,
                        header.getContext(),
                      )}
                </TableHead>
              ))}
            </TableRow>
          ))}
        </TableHeader>
        <TableBody>
          {table.getRowModel().rows.map((row) => (
            <TableRow key={row.id}>
              {row.getVisibleCells().map((cell) => (
                <TableCell key={cell.id}>
                  {flexRender(cell.column.columnDef.cell, cell.getContext())}
                </TableCell>
              ))}
            </TableRow>
          ))}
        </TableBody>
      </Table>

      {serverSidePagination
        ? totalPages &&
          totalPages > 1 && (
            <div className="mt-8">
              <Pagination>
                <PaginationContent>
                  {currentPage && currentPage > 1 && (
                    <PaginationItem>
                      <PaginationPrevious
                        href="#"
                        onClick={(e) => {
                          e.preventDefault();
                          onPageChange?.(currentPage - 1);
                        }}
                        disabled={isLoading}
                      >
                        {isLoading ? (
                          <ReloadIcon className="mr-2 h-4 w-4 animate-spin" />
                        ) : (
                          <ChevronLeftIcon className="mr-2 h-4 w-4" />
                        )}
                        Previous
                      </PaginationPrevious>
                    </PaginationItem>
                  )}
                  {currentPage &&
                    Array.from({ length: totalPages }, (_, i) => i + 1)
                      .filter((pageNumber) => {
                        if (totalPages <= 10) return true;
                        if (pageNumber === 1 || pageNumber === totalPages)
                          return true;
                        if (Math.abs(pageNumber - currentPage) <= 2)
                          return true;
                        return false;
                      })
                      .map((pageNumber, index, array) => (
                        <React.Fragment key={pageNumber}>
                          {index > 0 && array[index - 1] !== pageNumber - 1 && (
                            <PaginationItem>
                              <PaginationEllipsis />
                            </PaginationItem>
                          )}
                          <PaginationItem>
                            <PaginationLink
                              href="#"
                              onClick={(e) => {
                                e.preventDefault();
                                onPageChange?.(pageNumber);
                              }}
                              isActive={pageNumber === currentPage}
                              disabled={isLoading}
                            >
                              {pageNumber}
                            </PaginationLink>
                          </PaginationItem>
                        </React.Fragment>
                      ))}
                  {currentPage && currentPage < totalPages && (
                    <PaginationItem>
                      <PaginationNext
                        href="#"
                        onClick={(e) => {
                          e.preventDefault();
                          onPageChange?.(currentPage + 1);
                        }}
                        disabled={isLoading}
                      >
                        Next
                        {isLoading ? (
                          <ReloadIcon className="ml-2 h-4 w-4 animate-spin" />
                        ) : (
                          <ChevronRightIcon className="ml-2 h-4 w-4" />
                        )}
                      </PaginationNext>
                    </PaginationItem>
                  )}
                </PaginationContent>
              </Pagination>
            </div>
          )
        : table.getPageCount() > 1 && (
            <div className="mt-8">
              <Pagination>
                <PaginationContent>
                  {page > 1 && (
                    <PaginationItem>
                      <PaginationPrevious
                        href="#"
                        onClick={(e) => {
                          e.preventDefault();
                          handlePageChange(page - 1);
                        }}
                      >
                        <ChevronLeftIcon className="mr-2 h-4 w-4" />
                        Previous
                      </PaginationPrevious>
                    </PaginationItem>
                  )}
                  {Array.from({ length: table.getPageCount() }, (_, i) => i + 1)
                    .filter((pageNumber) => {
                      if (table.getPageCount() <= 10) return true;
                      if (
                        pageNumber === 1 ||
                        pageNumber === table.getPageCount()
                      )
                        return true;
                      if (Math.abs(pageNumber - page) <= 2) return true;
                      return false;
                    })
                    .map((pageNumber, index, array) => (
                      <React.Fragment key={pageNumber}>
                        {index > 0 && array[index - 1] !== pageNumber - 1 && (
                          <PaginationItem>
                            <PaginationEllipsis />
                          </PaginationItem>
                        )}
                        <PaginationItem>
                          <PaginationLink
                            href="#"
                            onClick={(e) => {
                              e.preventDefault();
                              handlePageChange(pageNumber);
                            }}
                            isActive={pageNumber === page}
                          >
                            {pageNumber}
                          </PaginationLink>
                        </PaginationItem>
                      </React.Fragment>
                    ))}
                  {page < table.getPageCount() && (
                    <PaginationItem>
                      <PaginationNext
                        href="#"
                        onClick={(e) => {
                          e.preventDefault();
                          handlePageChange(page + 1);
                        }}
                      >
                        Next
                        <ChevronRightIcon className="ml-2 h-4 w-4" />
                      </PaginationNext>
                    </PaginationItem>
                  )}
                </PaginationContent>
              </Pagination>
            </div>
          )}
    </div>
  );
}
