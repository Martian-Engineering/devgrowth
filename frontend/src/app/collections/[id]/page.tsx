// src/app/collections/[id]/page.tsx
"use client";

import { useState, useEffect, useCallback } from "react";
import { useParams } from "next/navigation";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogTrigger,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { ManageRepositoriesDialog } from "@/components/ManageRepositoriesDialog";
import { toast } from "@/hooks/use-toast";
import {
  MAUGrowthAccountingChart,
  MAUGrowthAccountingData,
} from "@/components/MAUGrowthAccountingChart";
import {
  MRRGrowthAccountingChart,
  MRRGrowthAccountingData,
} from "@/components/MRRGrowthAccountingChart";
import {
  CohortChart,
  ChartType,
  CohortDataItem,
} from "@/components/CohortChart";
import { addMonths, startOfMonth, endOfMonth, subYears } from "date-fns";
import { toZonedTime } from "date-fns-tz";
import { DateRange } from "react-day-picker";
import { DatePickerWithRange } from "@/components/ui/date-picker";
import { fetchWrapper } from "@/lib/fetchWrapper";
import { useProfile } from "@/contexts/ProfileContext";

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

interface GrowthAccountingResponse {
  mau_growth_accounting: MAUGrowthAccountingData[];
  mrr_growth_accounting: MRRGrowthAccountingData[];
  ltv_cumulative_cohort: CohortDataItem[];
}

export default function CollectionPage() {
  const { id } = useParams();
  const [collection, setCollection] = useState<Collection | null>(null);
  const [isManageReposDialogOpen, setIsManageReposDialogOpen] = useState(false);
  const [growthData, setGrowthData] = useState<GrowthAccountingResponse>({
    mau_growth_accounting: [],
    mrr_growth_accounting: [],
    ltv_cumulative_cohort: [],
  });
  const [filteredData, setFilteredData] = useState<GrowthAccountingResponse>({
    mau_growth_accounting: [],
    mrr_growth_accounting: [],
    ltv_cumulative_cohort: [],
  });
  const { profile, dispatch } = useProfile();

  const today = new Date();
  const lastMonth = addMonths(today, -1);
  const initialDateRange = {
    from: startOfMonth(subYears(lastMonth, 1)),
    to: endOfMonth(lastMonth),
  };

  const handleDateRangeChange = useCallback(
    (range: DateRange | undefined) => {
      if (range?.from && range?.to) {
        const {
          mau_growth_accounting,
          mrr_growth_accounting,
          ltv_cumulative_cohort,
        } = growthData;
        setFilteredData({
          mau_growth_accounting: mau_growth_accounting.filter((item) => {
            const itemDate = toZonedTime(new Date(item.month), "UTC");
            return itemDate >= range.from! && itemDate <= range.to!;
          }),
          mrr_growth_accounting: mrr_growth_accounting.filter((item) => {
            const itemDate = toZonedTime(new Date(item.month), "UTC");
            return itemDate >= range.from! && itemDate <= range.to!;
          }),
          ltv_cumulative_cohort: ltv_cumulative_cohort.filter((item) => {
            const itemDate = toZonedTime(new Date(item.first_month), "UTC");
            return itemDate >= range.from! && itemDate <= range.to!;
          }),
        });
      } else {
        setFilteredData(growthData);
      }
    },
    [growthData],
  );

  const handleRepositoriesChanged = () => {
    setIsManageReposDialogOpen(false);
    fetchGrowthAccountingData();
  };

  const fetchCollection = useCallback(async () => {
    try {
      const response = await fetchWrapper(`/api/collections/${id}`);
      if (!response.ok) throw new Error("Failed to fetch collection");
      const data = await response.json();
      setCollection(data);
    } catch (error) {
      console.error("Error fetching collection:", error);
      toast({
        title: "Error",
        description: "Failed to fetch collection. Please try again.",
        variant: "destructive",
      });
    }
  }, [id]);

  const fetchGrowthAccountingData = useCallback(async () => {
    try {
      const response = await fetchWrapper(`/api/collections/${id}/ga`);
      if (!response.ok)
        throw new Error("Failed to fetch growth accounting data");
      const data = await response.json();
      console.log(data);
      setGrowthData(data);
      setFilteredData(data);
    } catch (error) {
      console.error("Error fetching growth accounting data:", error);
      toast({
        title: "Error",
        description:
          "Failed to fetch growth accounting data. Please try again.",
        variant: "destructive",
      });
    }
  }, [id]);

  useEffect(() => {
    if (profile?.collections) {
      const collection = profile.collections.find(
        (collection) => collection.collection_id === Number(id),
      );
      if (collection) {
        setCollection(collection);
      }
    }
  }, [id, profile?.collections]);

  useEffect(() => {
    fetchCollection();
    fetchGrowthAccountingData();
  }, [id, fetchCollection, fetchGrowthAccountingData]);

  const handleRemoveRepository = async (repoId: number) => {
    try {
      const response = await fetchWrapper(
        `/api/collections/${id}/repositories/${repoId}`,
        {
          method: "DELETE",
        },
      );
      if (!response.ok) throw new Error("Failed to remove repository");
      dispatch({
        type: "REMOVE_REPOSITORY_FROM_COLLECTION",
        payload: { collectionId: Number(id), repoId: repoId },
      });
      fetchGrowthAccountingData();
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
    <div className="container mx-auto p-4 space-y-4">
      <h1 className="text-2xl font-bold mb-4">{collection.name}</h1>
      <p className="mb-4">{collection.description}</p>

      <Dialog
        open={isManageReposDialogOpen}
        onOpenChange={setIsManageReposDialogOpen}
      >
        <DialogTrigger asChild>
          <Button className="mt-4">Manage Repositories</Button>
        </DialogTrigger>
        <DialogContent className="sm:max-w-[825px]">
          <DialogHeader>
            <DialogTitle>Manage Collection Repositories</DialogTitle>
          </DialogHeader>
          <ManageRepositoriesDialog
            collectionId={collection?.collection_id || 0}
            onRepositoriesChanged={handleRepositoriesChanged}
            onClose={() => setIsManageReposDialogOpen(false)}
          />
        </DialogContent>
      </Dialog>

      {growthData &&
        growthData.mau_growth_accounting &&
        growthData.mau_growth_accounting.length > 0 && (
          <>
            <DatePickerWithRange
              className="w-[300px]"
              initialDateRange={initialDateRange}
              onDateRangeChange={handleDateRangeChange}
            />
            <MAUGrowthAccountingChart
              data={filteredData.mau_growth_accounting}
            />
            <MRRGrowthAccountingChart
              data={filteredData.mrr_growth_accounting}
            />
            <CohortChart
              data={filteredData.ltv_cumulative_cohort}
              chartType={ChartType.LogoRetention}
            />
            <CohortChart
              data={filteredData.ltv_cumulative_cohort}
              chartType={ChartType.CohortLTV}
            />
            <CohortChart
              data={filteredData.ltv_cumulative_cohort}
              chartType={ChartType.CommitRetention}
            />
          </>
        )}

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
