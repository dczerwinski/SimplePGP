use std::cell::RefCell;
use std::rc::Rc;

use crate::models::PgpKey;
use crate::services::GpgService;
use crate::utils::spawn_blocking;

pub type KeyListCallback = Box<dyn Fn(&KeyListState)>;

#[derive(Debug, Clone)]
pub struct KeyListState {
    pub keys: Vec<PgpKey>,
    pub selected_index: Option<usize>,
    pub loading: bool,
    pub error: Option<String>,
    pub import_result: Option<Result<String, String>>,
}

impl Default for KeyListState {
    fn default() -> Self {
        Self {
            keys: Vec::new(),
            selected_index: None,
            loading: false,
            error: None,
            import_result: None,
        }
    }
}

impl KeyListState {
    pub fn selected_key(&self) -> Option<&PgpKey> {
        self.selected_index.and_then(|i| self.keys.get(i))
    }
}

/// ViewModel for the key list. Holds state and notifies listeners on changes.
pub struct KeyListViewModel {
    state: Rc<RefCell<KeyListState>>,
    listeners: Rc<RefCell<Vec<KeyListCallback>>>,
}

impl KeyListViewModel {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            state: Rc::new(RefCell::new(KeyListState::default())),
            listeners: Rc::new(RefCell::new(Vec::new())),
        })
    }

    pub fn subscribe(&self, callback: KeyListCallback) {
        self.listeners.borrow_mut().push(callback);
    }

    pub fn state(&self) -> KeyListState {
        self.state.borrow().clone()
    }

    fn notify(&self) {
        let state = self.state.borrow().clone();
        for listener in self.listeners.borrow().iter() {
            listener(&state);
        }
    }

    fn update<F: FnOnce(&mut KeyListState)>(&self, f: F) {
        f(&mut self.state.borrow_mut());
        self.notify();
    }

    pub fn load_keys(self: &Rc<Self>) {
        self.update(|s| {
            s.loading = true;
            s.error = None;
        });

        let vm = Rc::clone(self);
        spawn_blocking(
            move || {
                let svc = GpgService::new();
                svc.list_all_keys()
            },
            move |result| match result {
                Ok(keys) => {
                    vm.update(|s| {
                        s.keys = keys;
                        s.loading = false;
                        s.selected_index = None;
                    });
                }
                Err(e) => {
                    vm.update(|s| {
                        s.loading = false;
                        s.error = Some(format!("Failed to load keys: {}", e));
                    });
                }
            },
        );
    }

    pub fn select_key(&self, index: Option<usize>) {
        self.update(|s| {
            s.selected_index = index;
        });
    }

    pub fn import_key(self: &Rc<Self>, armored_key: String) {
        self.update(|s| {
            s.loading = true;
            s.import_result = None;
        });

        let vm = Rc::clone(self);
        spawn_blocking(
            move || {
                let svc = GpgService::new();
                svc.import_key(&armored_key)
            },
            move |result| {
                let import_res = match &result {
                    Ok(msg) => Ok(msg.clone()),
                    Err(e) => Err(e.to_string()),
                };
                vm.update(|s| {
                    s.loading = false;
                    s.import_result = Some(import_res);
                });
                // Auto-refresh after import
                if result.is_ok() {
                    vm.load_keys();
                }
            },
        );
    }

    pub fn clear_import_result(&self) {
        self.update(|s| {
            s.import_result = None;
        });
    }
}
