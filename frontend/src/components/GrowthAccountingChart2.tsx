// components/GrowthAccountingChart2.tsx
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
} from "chart.js";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ChartContainer } from "@/components/ui/chart";

ChartJS.register(
  CategoryScale,
  LinearScale,
  BarElement,
  Title,
  Tooltip,
  Legend,
);

interface GrowthAccountingData {
  date: string;
  mau: number;
  retained: number;
  new: number;
  resurrected: number;
  churned: number;
}

interface GrowthAccountingChartProps {
  data: GrowthAccountingData[];
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

export function GrowthAccountingChart({ data }: GrowthAccountingChartProps) {
  const chartData: ChartData<"bar"> = {
    labels: data.map((item) => item.date),
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
      },
      y: {
        stacked: true,
      },
    },
    plugins: {
      tooltip: {
        callbacks: {
          label: (context) => {
            const label = context.dataset.label || "";
            const value = context.parsed.y;
            return `${label}: ${Math.abs(value)}`;
          },
        },
      },
    },
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>Growth Accounting</CardTitle>
      </CardHeader>
      <CardContent>
        <ChartContainer config={chartConfig} className="h-[400px]">
          <Bar data={chartData} options={options} />
        </ChartContainer>
      </CardContent>
    </Card>
  );
}
