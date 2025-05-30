import { useState } from 'react';

import { useParams } from '@tanstack/react-router';
import { Columns, Table } from 'lucide-react';

import { Button } from '@/components/ui/button';
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from '@/components/ui/resizable';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs2';
import { TableDataUploadDialog } from '@/modules/shared/table-data-upload-dialog/table-data-upload-dialog';
import { useGetTableColumns, useGetTablePreviewData } from '@/orval/tables';

import { DataPageContent } from '../shared/data-page/data-page-content';
import { DataPageHeader } from '../shared/data-page/data-page-header';
import { DataPageTrees } from '../shared/data-page/data-page-trees';
import { DataPreviewTable } from '../shared/data-preview-table/data-preview-table';
import { ColumnsTable } from './columns-page-table';

export function ColumnsPage() {
  const [isLoadDataDialogOpened, setIsLoadDataDialogOpened] = useState(false);
  const { databaseName, schemaName, tableName } = useParams({
    from: '/databases/$databaseName/schemas/$schemaName/tables/$tableName/columns/',
  });
  const { data: { items: columns } = {}, isFetching } = useGetTableColumns(
    databaseName,
    schemaName,
    tableName,
  );

  const { data: { items: previewData } = {}, isFetching: isPreviewDataFetching } =
    useGetTablePreviewData(databaseName, schemaName, tableName);

  return (
    <>
      <ResizablePanelGroup direction="horizontal">
        <ResizablePanel collapsible defaultSize={20} minSize={20} order={1}>
          <DataPageTrees />
        </ResizablePanel>
        <ResizableHandle withHandle />
        <ResizablePanel collapsible defaultSize={20} order={1}>
          <DataPageHeader
            title={tableName}
            Icon={Table}
            secondaryText={`${columns?.length} columns found`}
            Action={
              <Button onClick={() => setIsLoadDataDialogOpened(true)} disabled={isFetching}>
                Load Data
              </Button>
            }
          />
          <Tabs defaultValue="columns" className="size-full">
            <TabsList className="px-4">
              <TabsTrigger value="columns">Columns</TabsTrigger>
              <TabsTrigger value="data-preview">Data Preview</TabsTrigger>
            </TabsList>
            <TabsContent value="columns" className="m-0">
              <DataPageContent
                hasTabs
                isEmpty={!columns?.length}
                Table={<ColumnsTable isLoading={isFetching} columns={columns ?? []} />}
                emptyStateIcon={Columns}
                emptyStateTitle="No Columns Found"
                emptyStateDescription="No columns have been created yet. Create a column to get started."
              />
            </TabsContent>
            <TabsContent value="data-preview" className="m-0">
              <DataPageContent
                hasTabs
                isEmpty={!previewData?.length}
                Table={
                  <DataPreviewTable isLoading={isPreviewDataFetching} columns={previewData ?? []} />
                }
                emptyStateIcon={Columns}
                emptyStateTitle="No Data Found"
                emptyStateDescription="No data has been loaded for this table yet."
              />
            </TabsContent>
          </Tabs>
        </ResizablePanel>
      </ResizablePanelGroup>
      {databaseName && schemaName && tableName && (
        <TableDataUploadDialog
          opened={isLoadDataDialogOpened}
          onSetOpened={setIsLoadDataDialogOpened}
          databaseName={databaseName}
          schemaName={schemaName}
          tableName={tableName}
        />
      )}
    </>
  );
}
