// components/MAUGrowthAccountingChart.tsx
"use client";

import React from "react";
import { Bar } from "react-chartjs-2";
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  BarElement,
  Title,
  Tooltip,
  Legend,
  ChartData,
  ChartOptions,
  TimeSeriesScale,
} from "chart.js";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ChartContainer } from "@/components/ui/chart";
import { parse, format } from "date-fns";
import "chartjs-adapter-date-fns";
import { toZonedTime } from "date-fns-tz";

ChartJS.register(
  CategoryScale,
  LinearScale,
  BarElement,
  Title,
  Tooltip,
  Legend,
  TimeSeriesScale,
);

export interface MAUGrowthAccountingData {
  month: Date;
  mau: number;
  retained: number;
  new: number;
  resurrected: number;
  churned: number;
}

interface MAUGrowthAccountingChartProps {
  data: MAUGrowthAccountingData[];
}

const chartConfig = {
  new: {
    label: "New",
    color: "#0080ff",
  },
  retained: {
    label: "Retained",
    color: "#339966",
  },
  resurrected: {
    label: "Resurrected",
    color: "#f9a03f",
  },
  churned: {
    label: "Churned",
    color: "#e66666",
  },
};

export function MAUGrowthAccountingChart({
  data,
}: MAUGrowthAccountingChartProps) {
  const chartData: ChartData<"bar"> = {
    labels: data.map((item) => item.month),
    datasets: [
      {
        label: chartConfig.new.label,
        data: data.map((item) => item.new),
        backgroundColor: chartConfig.new.color,
      },
      {
        label: chartConfig.resurrected.label,
        data: data.map((item) => item.resurrected),
        backgroundColor: chartConfig.resurrected.color,
      },
      {
        label: chartConfig.retained.label,
        data: data.map((item) => item.retained),
        backgroundColor: chartConfig.retained.color,
      },
      {
        label: chartConfig.churned.label,
        data: data.map((item) => item.churned), // Already negative, no need to negate
        backgroundColor: chartConfig.churned.color,
      },
    ],
  };

  const options: ChartOptions<"bar"> = {
    responsive: true,
    maintainAspectRatio: false,
    scales: {
      x: {
        stacked: true,
        type: "time",
        time: {
          unit: "month",
          displayFormats: {
            month: "MM-yyyy",
          },
        },
        ticks: {
          source: "data",
          autoSkip: false,
          callback: function (value, index) {
            const date = toZonedTime(new Date(data[index].month), "UTC");
            return format(date, "MM-yyyy");
          },
        },
      },
      y: {
        stacked: true,
      },
    },
    plugins: {
      tooltip: {
        callbacks: {
          title: (context) => {
            const dateString = context[0].label as string;
            const date = toZonedTime(
              parse(dateString, "MMM d, yyyy, h:mm:ss a", new Date()),
              "UTC",
            );
            return format(date, "MMM yyyy");
          },
          label: (context) => {
            const label = context.dataset.label || "";
            const value = context.parsed.y;
            const dataIndex = context.dataIndex;
            const mau = data[dataIndex].mau; // Assuming your data array is in scope
            return `${label}: ${Math.abs(value)} (MAU: ${mau})`;
          },
        },
      },
    },
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>Monthly Active Developers</CardTitle>
      </CardHeader>
      <CardContent>
        <ChartContainer config={chartConfig} className="h-[400px]">
          <Bar data={chartData} options={options} />
        </ChartContainer>
      </CardContent>
    </Card>
  );
}
