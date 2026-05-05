//! Gateway-backed tag write sink for Python scripts.

use openwebhmi_audit_log::{AuditEvent, AuditLog, WriteSource};
use openwebhmi_driver_api::TagAddress;
use openwebhmi_protocol::{Quality, TagPath, TagValue};
use openwebhmi_scripting::{TagWriteError, TagWriteSink};
use openwebhmi_tag_engine::TagStore;
use tracing::warn;

use crate::project::{DriverHandles, WriteCommand, WriteEnqueueError};

/// Routes script-driven tag writes through driver queues or memory tags.
#[derive(Clone)]
pub struct GatewayTagWriteSink {
    driver_handles: DriverHandles,
    store: TagStore,
    audit_log: Option<AuditLog>,
}

impl GatewayTagWriteSink {
    /// Create a sink from the gateway's active driver handles and tag store.
    pub fn new(driver_handles: DriverHandles, store: TagStore) -> Self {
        Self::with_audit_log(driver_handles, store, None)
    }

    /// Create a sink with optional security audit logging.
    pub fn with_audit_log(
        driver_handles: DriverHandles,
        store: TagStore,
        audit_log: Option<AuditLog>,
    ) -> Self {
        Self {
            driver_handles,
            store,
            audit_log,
        }
    }

    fn audit(&self, path: &TagPath, value: TagValue, success: bool, error: Option<String>) {
        let Some(audit_log) = &self.audit_log else {
            return;
        };
        if let Err(err) = audit_log.append(
            Some("script".to_string()),
            None,
            None,
            AuditEvent::TagWrite {
                path: path.clone(),
                value,
                success,
                source: WriteSource::Script,
                error,
            },
        ) {
            warn!(error = %err, "failed to append script tag-write audit event");
        }
    }
}

impl TagWriteSink for GatewayTagWriteSink {
    fn enqueue(&self, path: &TagPath, value: TagValue) -> Result<(), TagWriteError> {
        let Some((driver_id, address)) = split_tag_path(path) else {
            self.store.publish(path, value.clone(), Quality::Good);
            self.audit(path, value, true, None);
            return Ok(());
        };

        let Some(driver) = self.driver_handles.get(driver_id) else {
            self.store.publish(path, value.clone(), Quality::Good);
            self.audit(path, value, true, None);
            return Ok(());
        };

        match driver.try_write(WriteCommand {
            address: TagAddress::new(address.to_string()),
            value: value.clone(),
        }) {
            Ok(()) => {
                self.audit(path, value, true, None);
                Ok(())
            }
            Err(err) => {
                let write_error = match err {
                    WriteEnqueueError::Busy => TagWriteError::Busy(path.to_string()),
                    WriteEnqueueError::Closed => TagWriteError::Closed(path.to_string()),
                };
                self.audit(path, value, false, Some(write_error.to_string()));
                Err(write_error)
            }
        }
    }
}

fn split_tag_path(path: &str) -> Option<(&str, &str)> {
    let (driver_id, address) = path.split_once('/')?;
    (!driver_id.is_empty() && !address.is_empty()).then_some((driver_id, address))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_paths_publish_to_store() {
        let store = TagStore::new();
        let sink = GatewayTagWriteSink::new(DriverHandles::new(), store.clone());

        sink.enqueue(&TagPath::new("mem/derived"), TagValue::Real(42.0))
            .unwrap();

        assert_eq!(
            store.get("mem/derived").unwrap().value,
            TagValue::Real(42.0)
        );
    }

    #[test]
    fn script_writes_append_audit_events() {
        let audit_log = AuditLog::memory().unwrap();
        let store = TagStore::new();
        let sink = GatewayTagWriteSink::with_audit_log(
            DriverHandles::new(),
            store,
            Some(audit_log.clone()),
        );

        sink.enqueue(&TagPath::new("mem/derived"), TagValue::Real(42.0))
            .unwrap();

        let (entries, total) = audit_log
            .query(&openwebhmi_audit_log::AuditQuery::default())
            .unwrap();
        assert_eq!(total, 1);
        assert!(matches!(
            &entries[0].kind,
            AuditEvent::TagWrite {
                source: WriteSource::Script,
                success: true,
                ..
            }
        ));
    }
}
