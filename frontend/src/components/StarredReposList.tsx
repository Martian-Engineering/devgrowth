// components/StarredReposList.tsx
import { useState, useEffect } from "react";
import {
  Pagination,
  PaginationContent,
  PaginationItem,
  PaginationLink,
  PaginationNext,
  PaginationPrevious,
} from "@/components/ui/pagination";
import { useProfile, GithubRepo } from "@/contexts/ProfileContext";
import { GithubRepoCard } from "@/components/GithubRepoCard";

const ITEMS_PER_PAGE = 10;

interface StarredReposListProps {
  repos: GithubRepo[];
  onCollectionUpdate: () => void;
}

export function StarredReposList({
  repos,
  onCollectionUpdate,
}: StarredReposListProps) {
  const [currentPage, setCurrentPage] = useState(1);
  const { profile } = useProfile();

  const [totalPages, setTotalPages] = useState(0);
  const [paginatedRepos, setPaginatedRepos] = useState<GithubRepo[]>([]);

  useEffect(() => {
    setTotalPages(Math.ceil(repos.length / ITEMS_PER_PAGE));
    setPaginatedRepos(
      repos.slice(
        (currentPage - 1) * ITEMS_PER_PAGE,
        currentPage * ITEMS_PER_PAGE,
      ),
    );
  }, [currentPage, repos]);

  return (
    <div>
      <h2 className="text-2xl font-semibold mb-4">Your Starred Repositories</h2>
      {paginatedRepos.map((repo) => (
        <GithubRepoCard
          key={repo.id}
          repo={repo}
          collections={profile?.collections || []}
          onCollectionUpdate={onCollectionUpdate}
        />
      ))}

      {totalPages > 1 && (
        <Pagination>
          <PaginationContent>
            {currentPage > 1 ? (
              <PaginationItem>
                <PaginationPrevious
                  onClick={() =>
                    setCurrentPage((prev) => Math.max(prev - 1, 1))
                  }
                />
              </PaginationItem>
            ) : (
              <PaginationItem>
                <PaginationPrevious className="pointer-events-none opacity-50" />
              </PaginationItem>
            )}
            {[...Array(totalPages)].map((_, index) => (
              <PaginationItem key={index}>
                <PaginationLink
                  onClick={() => setCurrentPage(index + 1)}
                  isActive={currentPage === index + 1}
                >
                  {index + 1}
                </PaginationLink>
              </PaginationItem>
            ))}
            {currentPage < totalPages ? (
              <PaginationItem>
                <PaginationNext
                  onClick={() =>
                    setCurrentPage((prev) => Math.min(prev + 1, totalPages))
                  }
                />
              </PaginationItem>
            ) : (
              <PaginationItem>
                <PaginationNext className="pointer-events-none opacity-50" />
              </PaginationItem>
            )}
          </PaginationContent>
        </Pagination>
      )}
    </div>
  );
}
