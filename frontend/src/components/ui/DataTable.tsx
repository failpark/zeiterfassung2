import React, { useState } from 'react';
import { cn } from '../../utils/ui-utils';

interface Column<T> {
  header: string;
  accessorKey: keyof T | ((item: T) => React.ReactNode);
  className?: string;
}

interface Pagination {
  pageIndex: number;
  pageSize: number;
  pageCount: number;
  totalItems: number;
}

export interface DataTableProps<T> {
  data: T[];
  columns: Column<T>[];
  pagination?: Pagination;
  onPageChange?: (page: number) => void;
  onRowClick?: (item: T) => void;
  isLoading?: boolean;
  noDataMessage?: string;
  className?: string;
  tableClassName?: string;
  theadClassName?: string;
  tbodyClassName?: string;
  rowClassName?: (item: T) => string;
}

function DataTable<T>({
  data,
  columns,
  pagination,
  onPageChange,
  onRowClick,
  isLoading = false,
  noDataMessage = 'No data available',
  className,
  tableClassName,
  theadClassName,
  tbodyClassName,
  rowClassName,
}: DataTableProps<T>) {
  // Function to get value from accessor key or function
  const getValue = (item: T, accessor: Column<T>['accessorKey']) => {
    if (typeof accessor === 'function') {
      return accessor(item);
    }
    return item[accessor];
  };

  // Render loading skeleton
  if (isLoading) {
    return (
      <div className={cn('w-full overflow-auto', className)}>
        <table className={cn('w-full caption-bottom text-sm', tableClassName)}>
          <thead className={cn('border-b bg-muted/50', theadClassName)}>
            <tr>
              {columns.map((column, index) => (
                <th
                  key={index}
                  className="h-12 px-4 text-left align-middle font-medium text-muted-foreground"
                >
                  {column.header}
                </th>
              ))}
            </tr>
          </thead>
          <tbody className={cn('', tbodyClassName)}>
            {Array.from({ length: 5 }).map((_, rowIndex) => (
              <tr key={rowIndex} className="border-b transition-colors hover:bg-muted/50">
                {columns.map((_, colIndex) => (
                  <td key={colIndex} className="p-4">
                    <div className="h-5 w-full animate-pulse rounded bg-muted"></div>
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    );
  }

  // If no data
  if (data.length === 0) {
    return (
      <div className="flex w-full items-center justify-center p-8 text-center">
        <p className="text-muted-foreground">{noDataMessage}</p>
      </div>
    );
  }

  return (
    <div className={cn('w-full', className)}>
      <div className="rounded-md border">
        <table className={cn('w-full caption-bottom text-sm', tableClassName)}>
          <thead className={cn('border-b bg-muted/50', theadClassName)}>
            <tr>
              {columns.map((column, index) => (
                <th
                  key={index}
                  className={cn(
                    'h-12 px-4 text-left align-middle font-medium text-muted-foreground',
                    column.className
                  )}
                >
                  {column.header}
                </th>
              ))}
            </tr>
          </thead>
          <tbody className={cn('', tbodyClassName)}>
            {data.map((item, rowIndex) => (
              <tr
                key={rowIndex}
                className={cn(
                  'border-b transition-colors hover:bg-muted/50',
                  onRowClick && 'cursor-pointer',
                  rowClassName && rowClassName(item)
                )}
                onClick={() => onRowClick && onRowClick(item)}
              >
                {columns.map((column, colIndex) => (
                  <td
                    key={colIndex}
                    className={cn('p-4 align-middle', column.className)}
                  >
                    {getValue(item, column.accessorKey)}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {pagination && (
        <div className="flex items-center justify-between p-4">
          <div className="text-sm text-muted-foreground">
            Showing {pagination.pageIndex * pagination.pageSize + 1} to{' '}
            {Math.min((pagination.pageIndex + 1) * pagination.pageSize, pagination.totalItems)} of{' '}
            {pagination.totalItems} entries
          </div>
          <div className="flex items-center space-x-2">
            <button
              className="inline-flex items-center justify-center rounded-md px-3 py-2 text-sm font-medium ring-offset-background transition-colors hover:bg-muted focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50"
              onClick={() => onPageChange && onPageChange(pagination.pageIndex - 1)}
              disabled={pagination.pageIndex === 0}
            >
              Previous
            </button>
            {Array.from({ length: Math.min(5, pagination.pageCount) }, (_, i) => {
              // Show pages around current page
              const pageNums = [];
              const startPage = Math.max(0, pagination.pageIndex - 2);
              const endPage = Math.min(pagination.pageCount - 1, startPage + 4);
              
              for (let p = startPage; p <= endPage; p++) {
                pageNums.push(p);
              }
              
              return pageNums.map(pageNum => (
                <button
                  key={pageNum}
                  className={cn(
                    'inline-flex h-10 w-10 items-center justify-center rounded-md text-sm font-medium transition-colors hover:bg-muted focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2',
                    pagination.pageIndex === pageNum && 'bg-muted font-semibold'
                  )}
                  onClick={() => onPageChange && onPageChange(pageNum)}
                >
                  {pageNum + 1}
                </button>
              ));
            })}
            <button
              className="inline-flex items-center justify-center rounded-md px-3 py-2 text-sm font-medium ring-offset-background transition-colors hover:bg-muted focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50"
              onClick={() => onPageChange && onPageChange(pagination.pageIndex + 1)}
              disabled={pagination.pageIndex === pagination.pageCount - 1}
            >
              Next
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

export default DataTable;