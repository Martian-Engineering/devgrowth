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
import { Checkbox } from "@/components/ui/checkbox";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuItem,
} from "@/components/ui/dropdown-menu";
import { MoreHorizontal } from "lucide-react";
import { GithubRepo, useProfile } from "@/contexts/ProfileContext";

interface ManageRepositoriesTableProps {
  collectionId: number;
  repositories: GithubRepo[];
  onSelectionChange: (selectedRepos: GithubRepo[]) => void;
}

export function ManageRepositoriesTable({
  collectionId,
  repositories,
  onSelectionChange,
}: ManageRepositoriesTableProps) {
  const { profile } = useProfile();
  const [selection, setSelection] = useState<Set<number>>(new Set());

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
    onSelectionChange(repositories.filter((repo) => newSelection.has(repo.id)));
  };

  const columns: ColumnDef<GithubRepo>[] = [
    {
      id: "select",
      header: () => (
        <Checkbox
          checked={
            repositories.length > 0 && selection.size === repositories.length
          }
          onCheckedChange={(value) => {
            const newSelection: Set<number> = value
              ? new Set(repositories.map((repo) => repo.id))
              : new Set();
            setSelection(newSelection);
            onSelectionChange(
              repositories.filter((repo) => newSelection.has(repo.id)),
            );
          }}
          aria-label="Select all"
        />
      ),
      cell: ({ row }) => {
        const repo = row.original;
        return (
          <Checkbox
            checked={selection.has(repo.id)}
            onCheckedChange={() => toggleSelection(repo.id)}
            aria-label="Select row"
          />
        );
      },
    },
    {
      header: "Repository",
      accessorFn: (row: GithubRepo) => `${row.owner}/${row.name}`,
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
              onClick={() => window.open(row.original.html_url, "_blank")}
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
      <div className="flex items-center justify-between mt-4">
        <Button
          onClick={() => table.setPageIndex(0)}
          disabled={!table.getCanPreviousPage()}
        >
          {"<<"}
        </Button>
        <Button
          onClick={() => table.previousPage()}
          disabled={!table.getCanPreviousPage()}
        >
          {"<"}
        </Button>
        <Button
          onClick={() => table.nextPage()}
          disabled={!table.getCanNextPage()}
        >
          {">"}
        </Button>
        <Button
          onClick={() => table.setPageIndex(table.getPageCount() - 1)}
          disabled={!table.getCanNextPage()}
        >
          {">>"}
        </Button>
        <span>
          Page{" "}
          <strong>
            {table.getState().pagination.pageIndex + 1} of{" "}
            {table.getPageCount()}
          </strong>{" "}
        </span>
        <select
          value={table.getState().pagination.pageSize}
          onChange={(e) => {
            table.setPageSize(Number(e.target.value));
          }}
        >
          {[5, 10].map((pageSize) => (
            <option key={pageSize} value={pageSize}>
              Show {pageSize}
            </option>
          ))}
        </select>
      </div>
    </div>
  );
}
