// Copyright 2026 Lap contributors.
// SPDX-License-Identifier: Apache-2.0 OR MIT

/// Roll back a configured drag source unless startup transferred it to the session.
pub(crate) struct StartupCleanup<F: FnOnce()>(Option<F>);
impl<F: FnOnce()> StartupCleanup<F> {
    pub(crate) fn new(cleanup: F) -> Self {
        Self(Some(cleanup))
    }
    pub(crate) fn commit(mut self) {
        self.0.take();
    }
}
impl<F: FnOnce()> Drop for StartupCleanup<F> {
    fn drop(&mut self) {
        if let Some(cleanup) = self.0.take() {
            cleanup();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};

    fn start(
        source: Rc<RefCell<Vec<String>>>,
        target_present: bool,
        context_created: bool,
    ) -> Result<(), ()> {
        source
            .borrow_mut()
            .push("URI handler and drag source".into());
        let rollback_source = source.clone();
        let rollback = StartupCleanup::new(move || rollback_source.borrow_mut().clear());
        if !target_present || !context_created {
            return Err(());
        }
        rollback.commit();
        Ok(())
    }

    #[test]
    fn failed_startup_does_not_accumulate_handlers_on_retry() {
        let source = Rc::new(RefCell::new(Vec::new()));
        for (target_present, context_created) in [(false, false), (true, false)] {
            assert!(start(source.clone(), target_present, context_created).is_err());
            assert!(source.borrow().is_empty());
        }
        assert!(start(source.clone(), true, true).is_ok());
        assert_eq!(source.borrow().len(), 1);
    }
}
