
pub mod stubs {
    use std::sync::Mutex;

    use oauth2::{AccessToken, RefreshToken};

    use crate::commons::auth::token_db::TokenDB;
    use crate::social::Network;

    pub struct StubTokenDB {
        access_token: Mutex<AccessToken>,
        refresh_token: Mutex<RefreshToken>,
    }

    impl StubTokenDB {
        #[must_use]
        pub fn new() -> Self {
            StubTokenDB {
                access_token: Mutex::new(AccessToken::new(String::from("initial-access-token"))),
                refresh_token: Mutex::new(RefreshToken::new(String::from("initial-refresh-token"))),
            }
        }
    }

    impl Default for StubTokenDB {
        fn default() -> Self {
            Self::new()
        }
    }

    impl TokenDB for StubTokenDB {
        fn get_access_token(
            &self,
            _social_network: &Network,
        ) -> Result<oauth2::AccessToken, Box<dyn std::error::Error>> {
            let guard = self.access_token.lock().unwrap();
            Ok((*guard).clone())
        }

        fn get_refresh_token(
            &self,
            _social_network: &Network,
        ) -> Result<oauth2::RefreshToken, Box<dyn std::error::Error>> {
            let guard = self.refresh_token.lock().unwrap();
            Ok((*guard).clone())
        }

        fn store(
            &self,
            _social_network: &Network,
            access_token: &AccessToken,
            refresh_tokem: &RefreshToken,
        ) -> Result<(), Box<dyn std::error::Error>> {
            let mut guard = self.access_token.lock().unwrap();
            *guard = access_token.clone();

            let mut guard = self.refresh_token.lock().unwrap();
            *guard = refresh_tokem.clone();

            Ok(())
        }
    }
}