"use client";

import { useState, useEffect, useCallback } from "react";
import { useParams } from "next/navigation";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { GrowthAccountingChart } from "@/components/GrowthAccountingChart2";
import { addMonths, startOfMonth, endOfMonth, subYears } from "date-fns";
import { toZonedTime } from "date-fns-tz";
import { DateRange } from "react-day-picker";

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

interface GrowthAccountingData {
  month: Date;
  mau: number;
  retained: number;
  new: number;
  resurrected: number;
  churned: number;
}

export default function RepositoryPage() {
  const params = useParams();
  const [metadata, setMetadata] = useState<RepositoryMetadata | null>(null);
  const [growthData, setGrowthData] = useState<GrowthAccountingData[]>([]);
  const [filteredData, setFilteredData] = useState(growthData);

  const today = new Date();
  const lastMonth = addMonths(today, -1);
  const initialDateRange = {
    from: startOfMonth(subYears(lastMonth, 1)),
    to: endOfMonth(lastMonth),
  };

  const handleDateRangeChange = useCallback(
    (range: DateRange | undefined) => {
      if (range?.from && range?.to) {
        console.log(range.from, range.to);
        const filtered = growthData.filter((item) => {
          const itemDate = toZonedTime(new Date(item.month), "UTC");
          return itemDate >= range.from && itemDate <= range.to;
        });
        console.log("filtered data", filtered);
        setFilteredData(filtered);
      } else {
        setFilteredData(growthData);
      }
    },
    [growthData],
  );

  useEffect(() => {
    if (params.owner && params.name) {
      fetch(`/api/repositories/${params.owner}/${params.name}`)
        .then((response) => response.json())
        .then((data) => setMetadata(data));

      fetch(`/api/repositories/${params.owner}/${params.name}/ga`)
        .then((response) => response.json())
        .then((data) => {
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

      {growthData.length > 0 && (
        <GrowthAccountingChart
          data={filteredData}
          initialDateRange={initialDateRange}
          onDateRangeChange={handleDateRangeChange}
        />
      )}
    </div>
  );
}
