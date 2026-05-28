use crate::services::AuthService;

pub struct AuthServiceBuilder<S, P, T, B, E, M, V> {
    store: Option<S>,
    hasher: Option<P>,
    tokens: Option<T>,
    blacklist: Option<B>,
    email_sender: Option<E>,
    email_templates: Option<M>,
    ott_store: Option<V>,
}

#[allow(clippy::new_without_default)]
impl<S, P, T, B, E, M, V> AuthServiceBuilder<S, P, T, B, E, M, V> {
    pub fn new() -> Self {
        Self {
            store: None,
            hasher: None,
            tokens: None,
            blacklist: None,
            email_sender: None,
            email_templates: None,
            ott_store: None,
        }
    }

    pub fn store(mut self, store: S) -> Self {
        self.store = Some(store);
        self
    }

    pub fn hasher(mut self, hasher: P) -> Self {
        self.hasher = Some(hasher);
        self
    }

    pub fn tokens(mut self, tokens: T) -> Self {
        self.tokens = Some(tokens);
        self
    }

    pub fn blacklist(mut self, blacklist: B) -> Self {
        self.blacklist = Some(blacklist);
        self
    }

    pub fn email_sender(mut self, sender: E) -> Self {
        self.email_sender = Some(sender);
        self
    }

    pub fn email_templates(mut self, templates: M) -> Self {
        self.email_templates = Some(templates);
        self
    }

    pub fn ott_store(mut self, store: V) -> Self {
        self.ott_store = Some(store);
        self
    }

    pub fn build(self) -> AuthService<S, P, T, B, E, M, V> {
        AuthService {
            store: self.store.expect("store missing"),
            hasher: self.hasher.expect("hasher missing"),
            tokens: self.tokens.expect("tokens missing"),
            blacklist: self.blacklist.expect("blacklist missing"),
            email_sender: self.email_sender.expect("email_sender missing"),
            email_templates: self.email_templates.expect("email_templates missing"),
            ott_store: self.ott_store.expect("token_store missing"),
        }
    }
}
