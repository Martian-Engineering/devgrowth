// app/page.tsx
"use client";

import React, { useEffect, useState } from "react";
import { useSession } from "next-auth/react";

import Link from "next/link";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  Pagination,
  PaginationContent,
  PaginationEllipsis,
  PaginationItem,
  PaginationLink,
  PaginationNext,
  PaginationPrevious,
} from "@/components/ui/pagination";

import { useProfile } from "@/contexts/ProfileContext";
import { Repository } from "@/lib/repository";
import { fetchWrapper } from "@/lib/fetchWrapper";
import { RepositoryCard } from "@/components/RepositoryCard";
import { PaginatedResponse } from "@/types/PaginatedResponse";
import {
  ReloadIcon,
  ChevronLeftIcon,
  ChevronRightIcon,
} from "@radix-ui/react-icons";
import { Skeleton } from "@/components/ui/skeleton";

function Main() {
  const [repositories, setRepositories] = useState<Repository[]>([]);
  const [activeTab, setActiveTab] = useState("recent");
  const [page, setPage] = useState(1);
  const [isLoading, setIsLoading] = useState(false);
  const [totalPages, setTotalPages] = useState(1);
  const pageSize = 10; // repositories per page
  const { profile } = useProfile();

  const fetchRepositories = async (pageNumber: number) => {
    setIsLoading(true);
    try {
      const response = await fetchWrapper(
        `/api/repositories?page=${pageNumber}&pageSize=${pageSize}`,
      );
      if (!response.ok) throw new Error("Failed to fetch repositories");
      const data: PaginatedResponse<Repository> = await response.json();
      setRepositories(data.data);
      setTotalPages(data.total_pages);
    } catch (error) {
      console.error("Error fetching repositories:", error);
    } finally {
      console.log("here");
      setTimeout(() => {
        console.log("unset is loading");
        setIsLoading(false);
      }, 2000);
    }
  };

  useEffect(() => {
    fetchRepositories(page);
  }, [page]);

  const handlePageChange = (newPage: number) => {
    setPage(newPage);
    window.scrollTo(0, 0);
  };

  return (
    <div className="w-full max-w-6xl mx-auto">
      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList>
          <TabsTrigger value="recent">Recent</TabsTrigger>
          <TabsTrigger value="search">Search</TabsTrigger>
        </TabsList>
        <TabsContent value="recent">
          <h1 className="text-2xl font-bold mb-4">Recent Repositories</h1>
          <p className="text-muted-foreground mb-4">
            These repositories have recently been added to devgrowth.
          </p>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-2 gap-4">
            {repositories.map((repo) => (
              <RepositoryCard
                key={repo.repository_id}
                repo={repo}
                collections={profile?.collections || []}
              />
            ))}
          </div>
          {repositories.length > 0 && totalPages > 1 && (
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
                  {Array.from({ length: totalPages }, (_, i) => i + 1)
                    .filter((pageNumber) => {
                      if (totalPages <= 10) return true;
                      // Always show first and last page
                      if (pageNumber === 1 || pageNumber === totalPages)
                        return true;
                      // Show pages around current page
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
                            disabled={isLoading}
                          >
                            {pageNumber}
                          </PaginationLink>
                        </PaginationItem>
                      </React.Fragment>
                    ))}
                  {page < totalPages && (
                    <PaginationItem>
                      <PaginationNext
                        href="#"
                        onClick={(e) => {
                          e.preventDefault();
                          handlePageChange(page + 1);
                        }}
                        disabled={isLoading}
                      >
                        Next
                        {isLoading ? (
                          <ReloadIcon className="mr-2 h-4 w-4 animate-spin" />
                        ) : (
                          <ChevronRightIcon className="ml-2 h-4 w-4" />
                        )}
                      </PaginationNext>
                    </PaginationItem>
                  )}
                </PaginationContent>
              </Pagination>
            </div>
          )}
        </TabsContent>
      </Tabs>
    </div>
  );
}

export default function Home() {
  const { data: session, status } = useSession();
  const { profile, refetchProfile } = useProfile();

  useEffect(() => {
    if (status === "authenticated") {
      refetchProfile();
    }
  }, [status, refetchProfile]);

  if (status === "loading") {
    return (
      <div className="w-full max-w-6xl mx-auto">
        <div className="space-y-4">
          <div className="flex gap-4 items-center">
            <Skeleton className="h-10 w-20" /> {/* Tab skeleton */}
            <Skeleton className="h-10 w-20" />
          </div>
          <Skeleton className="h-8 w-48" /> {/* Heading skeleton */}
          <Skeleton className="h-4 w-96" /> {/* Subtitle skeleton */}
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-2 gap-4">
            {Array.from({ length: 4 }).map((_, i) => (
              <Skeleton
                key={i}
                className="h-32"
              /> /* Repository card skeletons */
            ))}
          </div>
          <div className="flex justify-center gap-2 mt-8">
            {Array.from({ length: 5 }).map((_, i) => (
              <Skeleton
                key={i}
                className="h-10 w-10"
              /> /* Pagination skeletons */
            ))}
          </div>
        </div>
      </div>
    );
  }
  return (
    <>
      {!session ? (
        <div className="text-center mt-4">
          <p className="mb-4">
            Please log in to view your starred repositories and collections
          </p>
          <Button asChild>
            <Link href="/api/auth/signin">Sign In</Link>
          </Button>
        </div>
      ) : profile ? (
        <Main />
      ) : (
        <div>Loading profile data...</div>
      )}
    </>
  );
}
