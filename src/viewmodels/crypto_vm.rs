use std::cell::RefCell;
use std::rc::Rc;

use crate::security::{clear_string, SecureString};
use crate::services::GpgService;
use crate::utils::spawn_blocking;

pub type CryptoCallback = Box<dyn Fn(&CryptoState)>;

#[derive(Debug, Clone)]
pub struct CryptoState {
    pub encrypt_output: String,
    pub decrypt_output: String,
    pub loading: bool,
    pub error: Option<String>,
}

impl Default for CryptoState {
    fn default() -> Self {
        Self {
            encrypt_output: String::new(),
            decrypt_output: String::new(),
            loading: false,
            error: None,
        }
    }
}

impl Drop for CryptoState {
    fn drop(&mut self) {
        clear_string(&mut self.decrypt_output);
    }
}

pub struct CryptoViewModel {
    state: Rc<RefCell<CryptoState>>,
    listeners: Rc<RefCell<Vec<CryptoCallback>>>,
}

impl CryptoViewModel {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            state: Rc::new(RefCell::new(CryptoState::default())),
            listeners: Rc::new(RefCell::new(Vec::new())),
        })
    }

    pub fn subscribe(&self, callback: CryptoCallback) {
        self.listeners.borrow_mut().push(callback);
    }

    #[allow(dead_code)]
    pub fn state(&self) -> CryptoState {
        self.state.borrow().clone()
    }

    fn notify(&self) {
        let state = self.state.borrow().clone();
        for listener in self.listeners.borrow().iter() {
            listener(&state);
        }
    }

    fn update<F: FnOnce(&mut CryptoState)>(&self, f: F) {
        f(&mut self.state.borrow_mut());
        self.notify();
    }

    pub fn encrypt(self: &Rc<Self>, plaintext: String, recipient_key_id: String) {
        if plaintext.is_empty() {
            self.update(|s| {
                s.error = Some("Please enter text to encrypt.".to_string());
            });
            return;
        }
        if recipient_key_id.is_empty() {
            self.update(|s| {
                s.error = Some("Please select a recipient key.".to_string());
            });
            return;
        }

        self.update(|s| {
            s.loading = true;
            s.error = None;
            s.encrypt_output.clear();
        });

        let vm = Rc::clone(self);
        spawn_blocking(
            move || {
                let svc = GpgService::new();
                svc.encrypt(&plaintext, &recipient_key_id)
            },
            move |result| match result {
                Ok(ciphertext) => {
                    vm.update(|s| {
                        s.encrypt_output = ciphertext;
                        s.loading = false;
                    });
                }
                Err(e) => {
                    vm.update(|s| {
                        s.loading = false;
                        s.error = Some(format!("Encryption failed: {}", e));
                    });
                }
            },
        );
    }

    pub fn decrypt(self: &Rc<Self>, ciphertext: String) {
        if ciphertext.is_empty() {
            self.update(|s| {
                s.error = Some("Please enter ciphertext to decrypt.".to_string());
            });
            return;
        }

        self.update(|s| {
            s.loading = true;
            s.error = None;
            clear_string(&mut s.decrypt_output);
        });

        let vm = Rc::clone(self);
        spawn_blocking(
            move || {
                let svc = GpgService::new();
                svc.decrypt(&ciphertext)
            },
            move |result: Result<SecureString, _>| match result {
                Ok(secure_plaintext) => {
                    let text = secure_plaintext.into_inner();
                    vm.update(|s| {
                        s.decrypt_output = text;
                        s.loading = false;
                    });
                }
                Err(e) => {
                    vm.update(|s| {
                        s.loading = false;
                        s.error = Some(format!("Decryption failed: {}", e));
                    });
                }
            },
        );
    }

    pub fn clear_encrypt(&self) {
        self.update(|s| {
            s.encrypt_output.clear();
            s.error = None;
        });
    }

    pub fn clear_decrypt(&self) {
        self.update(|s| {
            clear_string(&mut s.decrypt_output);
            s.error = None;
        });
    }
}
