pub mod geom_write;

use std::any::Any;
use std::fmt::{Debug, Display};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use datafusion::arrow::datatypes::SchemaRef;
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::error::Result;
use datafusion::execution::context::TaskContext;
use datafusion::physical_expr::{EquivalenceProperties, PhysicalSortExpr};
use datafusion::physical_plan::metrics::{BaselineMetrics, ExecutionPlanMetricsSet, MetricsSet};
use datafusion::physical_plan::{
    DisplayFormatType, ExecutionPlan, Partitioning, RecordBatchStream, SendableRecordBatchStream,
    Statistics,
};
use futures::{Stream, StreamExt};
use trace::debug;

pub type AssertExprRef = Arc<dyn AssertExpr>;

pub trait AssertExpr: Send + Sync + Display + Debug {
    fn assert(&self, batch: &RecordBatch) -> Result<()>;
}

/// Execution plan for a Expand
#[derive(Debug)]
pub struct AssertExec {
    assert_expr: AssertExprRef,
    /// The input plan
    child: Arc<dyn ExecutionPlan>,

    /// Execution metrics
    metrics: ExecutionPlanMetricsSet,
}

impl AssertExec {
    pub fn new(assert_expr: AssertExprRef, child: Arc<dyn ExecutionPlan>) -> Self {
        Self {
            assert_expr,
            child,
            metrics: ExecutionPlanMetricsSet::new(),
        }
    }
}

impl ExecutionPlan for AssertExec {
    /// Return a reference to Any that can be used for downcasting
    fn as_any(&self) -> &dyn Any {
        self
    }

    /// Get the schema for this execution plan
    fn schema(&self) -> SchemaRef {
        self.child.schema()
    }

    /// Specifies whether this plan generates an infinite stream of records.
    /// If the plan does not support pipelining, but it its input(s) are
    /// infinite, returns an error to indicate this.
    fn unbounded_output(&self, children: &[bool]) -> Result<bool> {
        self.child.unbounded_output(children)
    }

    fn children(&self) -> Vec<Arc<dyn ExecutionPlan>> {
        vec![self.child.clone()]
    }

    /// Get the output partitioning of this plan
    fn output_partitioning(&self) -> Partitioning {
        self.child.output_partitioning()
    }

    fn output_ordering(&self) -> Option<&[PhysicalSortExpr]> {
        self.child.output_ordering()
    }

    fn maintains_input_order(&self) -> Vec<bool> {
        // tell optimizer this operator doesn't reorder its input
        vec![true]
    }

    fn equivalence_properties(&self) -> EquivalenceProperties {
        self.child.equivalence_properties()
    }

    fn with_new_children(
        self: Arc<Self>,
        children: Vec<Arc<dyn ExecutionPlan>>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        Ok(Arc::new(AssertExec::new(
            self.assert_expr.clone(),
            children[0].clone(),
        )))
    }

    fn execute(
        &self,
        partition: usize,
        context: Arc<TaskContext>,
    ) -> Result<SendableRecordBatchStream> {
        debug!(
            "Start ExpandExec::execute for partition {} of context session_id {} and task_id {:?}",
            partition,
            context.session_id(),
            context.task_id()
        );

        Ok(Box::pin(AssertStream {
            assert_expr: self.assert_expr.clone(),
            input: self.child.execute(partition, context)?,
            baseline_metrics: BaselineMetrics::new(&self.metrics, partition),
        }))
    }

    fn fmt_as(&self, _: DisplayFormatType, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "AssertExec: exprs=[{}]", self.assert_expr)
    }

    fn metrics(&self) -> Option<MetricsSet> {
        Some(self.metrics.clone_inner())
    }

    fn statistics(&self) -> Statistics {
        // TODO stats: compute statistics from assert_expr
        Statistics::default()
    }
}

struct AssertStream {
    assert_expr: AssertExprRef,
    input: SendableRecordBatchStream,
    baseline_metrics: BaselineMetrics,
}

impl Stream for AssertStream {
    type Item = Result<RecordBatch>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let poll = self.input.poll_next_unpin(cx).map(|x| match x {
            Some(Ok(batch)) => match self.assert_expr.assert(&batch) {
                Ok(_) => Some(Ok(batch)),
                Err(err) => Some(Err(err)),
            },
            other => other,
        });

        self.baseline_metrics.record_poll(poll)
    }
}

impl RecordBatchStream for AssertStream {
    /// Get the schema
    fn schema(&self) -> SchemaRef {
        self.input.schema()
    }
}
