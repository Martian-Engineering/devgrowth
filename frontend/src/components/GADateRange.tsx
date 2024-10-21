import React, { useCallback } from "react";
import { addMonths, startOfMonth, endOfMonth, subYears } from "date-fns";
import { toZonedTime } from "date-fns-tz";
import { DateRange } from "react-day-picker";
import { DatePickerWithRange } from "@/components/ui/date-picker";

interface GADateRangeProps {
  onDateRangeChange: (filteredData: any) => void;
  growthData: any;
}

export const GADateRange: React.FC<GADateRangeProps> = ({
  onDateRangeChange,
  growthData,
}) => {
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
        const filteredData = {
          mau_growth_accounting: mau_growth_accounting.filter((item: any) => {
            const itemDate = toZonedTime(new Date(item.month), "UTC");
            return itemDate >= range.from! && itemDate <= range.to!;
          }),
          mrr_growth_accounting: mrr_growth_accounting.filter((item: any) => {
            const itemDate = toZonedTime(new Date(item.month), "UTC");
            return itemDate >= range.from! && itemDate <= range.to!;
          }),
          ltv_cumulative_cohort: ltv_cumulative_cohort.filter((item: any) => {
            const itemDate = toZonedTime(new Date(item.first_month), "UTC");
            return itemDate >= range.from! && itemDate <= range.to!;
          }),
        };
        onDateRangeChange(filteredData);
      } else {
        onDateRangeChange(growthData);
      }
    },
    [growthData, onDateRangeChange],
  );

  return (
    <DatePickerWithRange
      className="w-[300px]"
      initialDateRange={initialDateRange}
      onDateRangeChange={handleDateRangeChange}
    />
  );
};
