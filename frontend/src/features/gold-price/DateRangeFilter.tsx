type DateRangeFilterProps = {
  startDate: string;
  endDate: string;
  isLoading: boolean;
  isRefreshing: boolean;
  onStartDateChange: (value: string) => void;
  onEndDateChange: (value: string) => void;
  onSubmit: () => void;
  onRefresh: () => void;
};

export function DateRangeFilter({
  startDate,
  endDate,
  isLoading,
  isRefreshing,
  onStartDateChange,
  onEndDateChange,
  onSubmit,
  onRefresh,
}: DateRangeFilterProps) {
  return (
    <form
      onSubmit={(event) => {
        event.preventDefault();
        onSubmit();
      }}
    >
      <label>
        开始日期
        <input
          type="date"
          value={startDate}
          onChange={(event) => onStartDateChange(event.target.value)}
        />
      </label>
      <label>
        结束日期
        <input
          type="date"
          value={endDate}
          onChange={(event) => onEndDateChange(event.target.value)}
        />
      </label>
      <button type="submit" disabled={isLoading || isRefreshing}>
        {isLoading ? '查询中…' : '查询价格'}
      </button>
      <button type="button" disabled={isLoading || isRefreshing} onClick={onRefresh}>
        {isRefreshing ? '汇总中…' : '汇总数据'}
      </button>
    </form>
  );
}
