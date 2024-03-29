use datafusion::arrow::array::ArrayRef;
use datafusion::arrow::datatypes::DataType;
use datafusion::common::ScalarValue;
use datafusion::error::DataFusionError;
use datafusion::physical_plan::Accumulator;
use spi::{DFResult, QueryError};

use crate::extension::expr::aggregate_function::state_agg::StateAggData;
use crate::extension::expr::aggregate_function::AggResult;

/// An accumulator to compute the average
#[derive(Debug)]
pub struct StateAggAccumulator {
    state: IntermediateState,
    input_type: Vec<DataType>,
    compact: bool,
}

impl StateAggAccumulator {
    pub fn try_new(input_type: Vec<DataType>, compact: bool) -> DFResult<Self> {
        Ok(Self {
            state: Default::default(),
            input_type,
            compact,
        })
    }
}

impl Accumulator for StateAggAccumulator {
    fn update_batch(&mut self, values: &[ArrayRef]) -> DFResult<()> {
        trace::trace!("update_batch: {:?}", values);

        if values.is_empty() {
            return Ok(());
        }

        debug_assert!(
            values.len() == 2,
            "compact_state_agg can only take 2 param!"
        );

        let times = &values[0];
        let state_records = &values[1];

        (0..times.len()).try_for_each(|index| {
            let time_record = ScalarValue::try_from_array(times.as_ref(), index)?;
            let state_record = ScalarValue::try_from_array(state_records.as_ref(), index)?;

            self.state.push(time_record, state_record);

            Ok(())
        })
    }

    fn evaluate(&self) -> DFResult<ScalarValue> {
        let indices = self.state.sort_indices();

        let mut state_agg_data = StateAggData::new(
            self.input_type[0].clone(),
            self.input_type[1].clone(),
            self.compact,
        );

        for idx in indices {
            match (
                self.state.state_value_records.get(idx),
                self.state.time_records.get(idx),
            ) {
                (Some(state_value), Some(time)) => {
                    if time.is_null() {
                        continue;
                    }
                    // All records must be processed in time order
                    state_agg_data.handle_record(state_value.clone(), time.clone())?;
                }
                _ => {
                    return Err(DataFusionError::External(Box::new(QueryError::Internal {
                        reason:
                            "The internal state data of CompactStateAggAccumulator is inconsistent."
                                .to_string(),
                    })))
                }
            }
        }
        state_agg_data.finalize();

        let result_state = state_agg_data.to_scalar()?;

        trace::trace!(
            "CompactStateAggAccumulator evaluate result: {:?}",
            result_state
        );

        Ok(result_state)
    }

    fn size(&self) -> usize {
        std::mem::size_of_val(self)
    }

    fn state(&self) -> DFResult<Vec<ScalarValue>> {
        if self.state.is_empty() {
            return empty_intermediate_state(&self.input_type);
        }

        let state = self.state.to_state()?;

        trace::trace!("CompactStateAggAccumulator state: {:?}", state,);

        Ok(state)
    }

    fn merge_batch(&mut self, values: &[ArrayRef]) -> DFResult<()> {
        trace::trace!("merge_batch: {:?}", values);

        if values.is_empty() {
            return Ok(());
        }

        debug_assert!(
            values.len() == 2,
            "merge_batch of compact_state_agg can only take 2 param."
        );

        let state = IntermediateState::from_state(values)?;

        self.state.append(state);

        Ok(())
    }
}

fn empty_intermediate_state(state_type: &[DataType]) -> DFResult<Vec<ScalarValue>> {
    let result = state_type
        .iter()
        .map(|e| ScalarValue::new_list(Some(vec![]), e.clone()))
        .collect();

    Ok(result)
}

#[derive(Default, Debug)]
struct IntermediateState {
    time_records: Vec<ScalarValue>,
    state_value_records: Vec<ScalarValue>,
}

impl IntermediateState {
    fn push(&mut self, time_record: ScalarValue, state_record: ScalarValue) {
        self.time_records.push(time_record);
        self.state_value_records.push(state_record);
    }

    fn append(&mut self, state: IntermediateState) {
        self.time_records.extend(state.time_records);
        self.state_value_records.extend(state.state_value_records);
    }

    fn sort_indices(&self) -> Vec<usize> {
        let mut sort = self.time_records.iter().enumerate().collect::<Vec<_>>();
        sort.sort_unstable_by(|(_, a), (_, b)| unsafe { a.partial_cmp(b).unwrap_unchecked() });
        sort.iter().map(|(i, _)| *i).collect()
    }
}

impl IntermediateState {
    fn is_empty(&self) -> bool {
        self.time_records.is_empty() || self.state_value_records.is_empty()
    }

    fn from_state(values: &[ArrayRef]) -> DFResult<Self> {
        if values.is_empty() {
            return Ok(Self::default());
        }

        debug_assert!(values.len() == 2, "compact_state_agg can only take 2 param");

        let mut time_records = vec![];
        let mut state_value_records = vec![];

        let times = &values[0];
        let state_records = &values[1];

        debug_assert!(
            times.len() == state_records.len(),
            "all columns of the input must have the same length"
        );

        (0..times.len()).try_for_each(|index| {
            let scalar = ScalarValue::try_from_array(times.as_ref(), index)?;
            if let ScalarValue::List(Some(values), _) = scalar {
                time_records.extend(values);
            } else {
                return Err(DataFusionError::Internal(format!(
                    "compact_state_agg state must be non-null list, but found: {}",
                    scalar.get_datatype()
                )));
            }

            let scalar = ScalarValue::try_from_array(state_records.as_ref(), index)?;
            if let ScalarValue::List(Some(values), _) = scalar {
                state_value_records.extend(values);
            } else {
                return Err(DataFusionError::Internal(format!(
                    "compact_state_agg state must be non-null list, but found: {}",
                    scalar.get_datatype()
                )));
            }

            Ok(())
        })?;

        Ok(Self {
            time_records,
            state_value_records,
        })
    }

    fn to_state(&self) -> DFResult<Vec<ScalarValue>> {
        if self.is_empty() {
            return Err(DataFusionError::External(Box::new(QueryError::Internal {
                reason: "IntermediateState is empty".to_string(),
            })));
        }

        let time_data_type = self.time_records[0].get_datatype();
        let state_data_type = self.state_value_records[0].get_datatype();

        let time_state = ScalarValue::new_list(Some(self.time_records.clone()), time_data_type);
        let value_state =
            ScalarValue::new_list(Some(self.state_value_records.clone()), state_data_type);

        trace::trace!("time_state data type: {:?}", time_state.get_datatype());

        Ok(vec![time_state, value_state])
    }
}
