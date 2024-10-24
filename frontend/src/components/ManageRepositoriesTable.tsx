// devgrowth/frontend/src/components/ManageRepositoriesTable.tsx
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
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuItem,
} from "@/components/ui/dropdown-menu";
import { MoreHorizontal } from "lucide-react";
import { useProfile } from "@/contexts/ProfileContext";
import { Repository } from "@/lib/repository";
import {
  ChevronLeftIcon,
  ChevronRightIcon,
  ReloadIcon,
} from "@radix-ui/react-icons";

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

export function ManageRepositoriesTable({
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
                const url = `https://github.com/${name}/${owner}`;
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
