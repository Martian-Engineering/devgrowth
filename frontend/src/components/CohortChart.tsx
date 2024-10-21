// components/CohortChart.tsx
"use client";

import React, { useMemo, useEffect, useState } from "react";
import { Line } from "react-chartjs-2";
import { ChartContainer } from "@/components/ui/chart";
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend,
  ChartData,
  ChartOptions,
} from "chart.js";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { format, parseISO } from "date-fns";

ChartJS.register(
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend,
);

export interface CohortDataItem {
  first_month: string;
  active_month: string;
  months_since_first: number;
  users: number;
  cohort_num_users: number;
  retained_pctg: number;
  inc_amt: number;
  cum_amt: number;
  cum_amt_per_user: number;
}

export enum ChartType {
  LogoRetention,
  CohortLTV,
  CommitRetention,
}

interface CohortChartProps {
  data: CohortDataItem[];
  chartType: ChartType;
}

const generateColor = (index: number) => {
  const hue = (index * 137.508) % 360; // Use golden angle approximation
  return `hsl(${hue}, 70%, 50%)`;
};

const chartTitles = {
  [ChartType.LogoRetention]: "Developer Retention",
  [ChartType.CohortLTV]: "Cohort LTV",
  [ChartType.CommitRetention]: "Commit Retention",
};

function getInitialIncAmtByCohort(data: CohortDataItem[]): Map<string, number> {
  const initialIncAmtMap = new Map<string, number>();

  data.forEach((item) => {
    if (
      item.months_since_first === 0 &&
      !initialIncAmtMap.has(item.first_month)
    ) {
      initialIncAmtMap.set(item.first_month, item.inc_amt);
    }
  });

  return initialIncAmtMap;
}

export function CohortChart({ data, chartType }: CohortChartProps) {
  const [initialRev, setInitialRev] = useState<Map<string, number>>(new Map());

  useEffect(() => {
    setInitialRev(getInitialIncAmtByCohort(data));
  }, [data]);

  const cohortData = useMemo(() => {
    const cohorts = new Map<string, CohortDataItem[]>();

    data.forEach((item) => {
      const cohortKey = item.first_month;
      if (!cohorts.has(cohortKey)) {
        cohorts.set(cohortKey, []);
      }
      cohorts.get(cohortKey)!.push(item);
    });

    return Array.from(cohorts.entries()).map(([key, values]) => ({
      cohort: key,
      data: values.sort((a, b) => a.months_since_first - b.months_since_first),
    }));
  }, [data]);

  const getYValue = (item: CohortDataItem) => {
    switch (chartType) {
      case ChartType.LogoRetention:
        return item.retained_pctg * 100; // Convert to percentage
      case ChartType.CohortLTV:
        return item.cum_amt_per_user;
      case ChartType.CommitRetention:
        const initialAmount = initialRev.get(item.first_month) || 1;
        return (item.inc_amt / initialAmount) * 100; // Convert to percentage
    }
  };

  const chartData: ChartData<"line"> = {
    labels: Array.from(
      new Set(data.map((item) => item.months_since_first)),
    ).sort((a, b) => a - b),
    datasets: cohortData.map((cohort, index) => ({
      label: format(parseISO(cohort.cohort), "MMM yyyy"),
      data: cohort.data.map((item) => getYValue(item)),
      borderColor: generateColor(index),
      backgroundColor: generateColor(index),
      fill: false,
    })),
  };

  const options: ChartOptions<"line"> = {
    responsive: true,
    maintainAspectRatio: false,
    scales: {
      x: {
        title: {
          display: true,
          text: "Months Since First Activity",
        },
      },
      y: {
        title: {
          display: true,
          text: (() => {
            switch (chartType) {
              case ChartType.CohortLTV:
                return "Cumulative Commits per User";
              case ChartType.LogoRetention:
                return "Developer Retention";
              case ChartType.CommitRetention:
                return "Commit Retention";
              default:
                return "Percentage";
            }
          })(),
        },
        beginAtZero: true,
        ticks: {
          callback: (value) =>
            chartType === ChartType.CohortLTV ? `${value}` : `${value}%`,
        },
      },
    },
    plugins: {
      tooltip: {
        callbacks: {
          title: (context) => `Month ${context[0].label}`,
          label: (context) => {
            const label = context.dataset.label || "";
            const value = context.parsed.y;
            return `${label}: ${value.toFixed(0)}`;
          },
        },
      },
    },
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>{chartTitles[chartType]}</CardTitle>
      </CardHeader>
      <CardContent>
        <ChartContainer config={{}} className="h-[400px]">
          <Line data={chartData} options={options} />
        </ChartContainer>
      </CardContent>
    </Card>
  );
}
