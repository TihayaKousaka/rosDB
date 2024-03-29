use datafusion::common::tree_node::{TreeNode, TreeNodeVisitor, VisitRecursion};
use datafusion::common::Result;
use datafusion::error::DataFusionError;
use datafusion::logical_expr::{LogicalPlan, TableScan};
use spi::query::datasource::stream::StreamProviderRef;
use spi::query::logical_planner::QueryPlan;

use crate::data_source::source_downcast_adapter;
use crate::data_source::table_source::TableHandle;

pub fn extract_stream_providers(plan: &QueryPlan) -> Vec<StreamProviderRef> {
    let mut stream_providers = vec![];

    let _ = plan.df_plan.visit(&mut ExtractStreamProvider {
        stream_providers: &mut stream_providers,
    });

    stream_providers
}

pub struct ExtractStreamProvider<'a> {
    stream_providers: &'a mut Vec<StreamProviderRef>,
}

impl<'a> TreeNodeVisitor for ExtractStreamProvider<'a> {
    type N = LogicalPlan;

    fn pre_visit(&mut self, plan: &LogicalPlan) -> Result<VisitRecursion> {
        if let LogicalPlan::TableScan(TableScan { source, .. }) = plan {
            if let TableHandle::StreamProvider(s) = source_downcast_adapter(source)
                .map_err(|err| DataFusionError::External(Box::new(err)))?
                .table_handle()
            {
                self.stream_providers.push(s.clone());
            }
        }

        Ok(VisitRecursion::Continue)
    }
}
