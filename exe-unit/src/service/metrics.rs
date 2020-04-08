use crate::error::Error;
use crate::message::{GetMetrics, Shutdown};
use crate::metrics::error::MetricError;
use crate::metrics::{CpuMetric, MemMetric, Metric, MetricData, MetricReport, TimeMetric};
use crate::ExeUnitContext;
use actix::prelude::*;
use chrono::{DateTime, Utc};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

pub struct MetricsService {
    usage_vector: Vec<String>,
    metrics: HashMap<String, MetricProvider>,
}

impl MetricsService {
    pub fn try_new(
        ctx: &ExeUnitContext,
        backlog_limit: Option<usize>,
    ) -> Result<Self, MetricError> {
        let metrics: HashMap<String, MetricProvider> = vec![
            (
                CpuMetric::ID.to_string(),
                MetricProvider::new(
                    CpuMetric::default(),
                    backlog_limit.clone(),
                    ctx.agreement.usage_limits.get(CpuMetric::ID).cloned(),
                ),
            ),
            (
                MemMetric::ID.to_string(),
                MetricProvider::new(
                    MemMetric::default(),
                    backlog_limit.clone(),
                    ctx.agreement.usage_limits.get(MemMetric::ID).cloned(),
                ),
            ),
            (
                TimeMetric::ID.to_string(),
                MetricProvider::new(
                    TimeMetric::default(),
                    Some(1),
                    ctx.agreement.usage_limits.get(TimeMetric::ID).cloned(),
                ),
            ),
        ]
        .into_iter()
        .collect();

        if let Some(e) = ctx
            .agreement
            .usage_vector
            .iter()
            .find(|e| !metrics.contains_key(*e))
        {
            return Err(MetricError::Unsupported(e.to_string()));
        }

        Ok(MetricsService {
            usage_vector: ctx.agreement.usage_vector.clone(),
            metrics,
        })
    }

    pub fn metrics() -> Vec<&'static str> {
        vec![CpuMetric::ID, MemMetric::ID, TimeMetric::ID]
    }
}

impl Actor for MetricsService {
    type Context = Context<Self>;
}

impl Handler<Shutdown> for MetricsService {
    type Result = <Shutdown as Message>::Result;

    fn handle(&mut self, _: Shutdown, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop();
        Ok(())
    }
}

impl Handler<GetMetrics> for MetricsService {
    type Result = <GetMetrics as Message>::Result;

    fn handle(&mut self, _: GetMetrics, _: &mut Self::Context) -> Self::Result {
        let mut metrics = Vec::with_capacity(self.usage_vector.len());

        for name in self.usage_vector.iter() {
            let metric = self
                .metrics
                .get_mut(name)
                .ok_or(MetricError::Unsupported(name.to_string()))?;

            let report = metric.report();
            metric.log_report(report.clone());

            match report {
                MetricReport::Frame(data) => metrics.push(data),
                MetricReport::Error(error) => return Err(error.into()),
                MetricReport::LimitExceeded(data) => {
                    return Err(Error::UsageLimitExceeded(format!(
                        "{:?} exceeded the value of {:?}",
                        name, data
                    )))
                }
            }
        }

        Ok(metrics)
    }
}

struct MetricProvider {
    metric: Box<dyn Metric>,
    backlog: Arc<Mutex<VecDeque<(DateTime<Utc>, MetricReport)>>>,
    backlog_limit: Option<usize>,
    usage_limit: Option<MetricData>,
}

impl MetricProvider {
    pub fn new<M: Metric + 'static>(
        metric: M,
        backlog_limit: Option<usize>,
        usage_limit: Option<MetricData>,
    ) -> Self {
        MetricProvider {
            metric: Box::new(metric),
            backlog: Arc::new(Mutex::new(VecDeque::new())),
            backlog_limit,
            usage_limit,
        }
    }
}

impl MetricProvider {
    fn report(&mut self) -> MetricReport {
        if let Ok(data) = self.metric.peak() {
            if let Some(limit) = &self.usage_limit {
                if data > *limit {
                    return MetricReport::LimitExceeded(data);
                }
            }
        }

        match self.metric.frame() {
            Ok(data) => MetricReport::Frame(data),
            Err(error) => MetricReport::Error(error),
        }
    }

    fn log_report(&mut self, report: MetricReport) {
        let mut backlog = self.backlog.lock().unwrap();
        if let Some(limit) = self.backlog_limit {
            if backlog.len() == limit {
                backlog.pop_back();
            }
        }
        backlog.push_front((Utc::now(), report));
    }
}