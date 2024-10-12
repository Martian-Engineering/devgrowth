// components/GrowthAccountingChart.tsx
"use client";

import React from "react";
import {
  Bar,
  BarChart,
  XAxis,
  YAxis,
  Tooltip,
  Legend,
  ResponsiveContainer,
  ReferenceLine,
} from "recharts";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ChartContainer } from "@/components/ui/chart";

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
    color: "hsl(var(--chart-new))",
  },
  retained: {
    label: "Retained",
    color: "hsl(var(--chart-retained))",
  },
  resurrected: {
    label: "Resurrected",
    color: "hsl(var(--chart-resurrected))",
  },
  churned: {
    label: "Churned",
    color: "hsl(var(--chart-churned))",
  },
};

export function GrowthAccountingChart({ data }: GrowthAccountingChartProps) {
  const CustomTooltip = ({ active, payload, label }: any) => {
    if (active && payload && payload.length) {
      return (
        <div className="bg-white p-4 border rounded shadow">
          <p className="font-bold">{`Date: ${label}`}</p>
          {payload.map((pld: any) => (
            <p key={pld.name} style={{ color: pld.fill }}>
              {`${pld.name}: ${pld.name === "Churned" ? -pld.value : pld.value}`}
            </p>
          ))}
        </div>
      );
    }
    return null;
  };
  return (
    <Card>
      <CardHeader>
        <CardTitle>Growth Accounting</CardTitle>
      </CardHeader>
      <CardContent>
        <ChartContainer config={chartConfig} className="h-[400px]">
          <ResponsiveContainer width="100%" height="100%">
            <BarChart data={data}>
              <XAxis dataKey="date" />
              <YAxis />
              <Tooltip content={<CustomTooltip />} />
              <Legend />
              <ReferenceLine y={0} stroke="#000" />
              <Bar dataKey="new" stackId="a" fill={chartConfig.new.color} />
              <Bar
                dataKey="resurrected"
                stackId="a"
                fill={chartConfig.resurrected.color}
              />
              <Bar
                dataKey="retained"
                stackId="a"
                fill={chartConfig.retained.color}
              />
              <Bar dataKey="churned" fill={chartConfig.churned.color} />
            </BarChart>
          </ResponsiveContainer>
        </ChartContainer>
      </CardContent>
    </Card>
  );
}
