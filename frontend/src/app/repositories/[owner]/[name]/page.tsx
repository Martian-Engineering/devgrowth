"use client";

import { useState, useEffect, useCallback } from "react";
import { useParams } from "next/navigation";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
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
import { GADateRange } from "@/components/GADateRange";
import { fetchWrapper } from "@/lib/fetchWrapper";

interface RepositoryMetadata {
  id: number;
  owner: string;
  name: string;
  commit_count: number;
  latest_commit_date: string | null;
  latest_commit_author: string | null;
  indexed_at: string | null;
  github_url: string;
}

interface GrowthAccountingResponse {
  mau_growth_accounting: MAUGrowthAccountingData[];
  mrr_growth_accounting: MRRGrowthAccountingData[];
  ltv_cumulative_cohort: CohortDataItem[];
}

export default function RepositoryPage() {
  const params = useParams();
  const [metadata, setMetadata] = useState<RepositoryMetadata | null>(null);
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

  const handleDateRangeChange = useCallback(
    (filteredData: GrowthAccountingResponse) => {
      setFilteredData(filteredData);
    },
    [],
  );

  useEffect(() => {
    if (params.owner && params.name) {
      fetchWrapper(`/api/repositories/${params.owner}/${params.name}`)
        .then((response) => response.json())
        .then((data) => setMetadata(data));

      fetchWrapper(`/api/repositories/${params.owner}/${params.name}/ga`)
        .then((response) => response.json())
        .then((data) => {
          console.log(data);
          setGrowthData(data);
          setFilteredData(data);
        });
    }
  }, [params.owner, params.name]);

  if (!metadata) {
    return <div>Loading...</div>;
  }

  return (
    <div className="container mx-auto p-4 space-y-4">
      <Card>
        <CardHeader>
          <CardTitle>
            {metadata.owner}/{metadata.name}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p>Commit Count: {metadata.commit_count}</p>
          <p>Latest Commit Date: {metadata.latest_commit_date || "N/A"}</p>
          <p>Latest Commit Author: {metadata.latest_commit_author || "N/A"}</p>
          <p>Indexed At: {metadata.indexed_at || "Not indexed yet"}</p>
          <p>
            GitHub URL:{" "}
            <a
              href={metadata.github_url}
              target="_blank"
              rel="noopener noreferrer"
            >
              {metadata.github_url}
            </a>
          </p>
        </CardContent>
      </Card>

      {growthData &&
        growthData.mau_growth_accounting &&
        growthData.mau_growth_accounting.length > 0 && (
          <>
            <GADateRange
              onDateRangeChange={handleDateRangeChange}
              growthData={growthData}
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
    </div>
  );
}
