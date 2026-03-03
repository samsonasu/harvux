use anyhow::{Context, Result};
use oo7::Keyring;

const TOKEN_ATTR: &str = "harvest_access_token";
const ACCOUNT_ATTR: &str = "harvest_account_id";
const APP_LABEL: &str = "Harvux";

pub struct Credentials {
    pub access_token: String,
    pub account_id: String,
}

pub async fn store_credentials(access_token: &str, account_id: &str) -> Result<()> {
    let keyring = Keyring::new().await.context("Failed to open keyring")?;

    keyring
        .create_item(
            &format!("{APP_LABEL} - Access Token"),
            &vec![("application", APP_LABEL), ("type", TOKEN_ATTR)],
            access_token,
            true,
        )
        .await
        .context("Failed to store access token")?;

    keyring
        .create_item(
            &format!("{APP_LABEL} - Account ID"),
            &vec![("application", APP_LABEL), ("type", ACCOUNT_ATTR)],
            account_id,
            true,
        )
        .await
        .context("Failed to store account ID")?;

    Ok(())
}

pub async fn load_credentials() -> Result<Option<Credentials>> {
    let keyring = Keyring::new().await.context("Failed to open keyring")?;

    let token_items = keyring
        .search_items(&vec![("application", APP_LABEL), ("type", TOKEN_ATTR)])
        .await
        .context("Failed to search for access token")?;

    let account_items = keyring
        .search_items(&vec![("application", APP_LABEL), ("type", ACCOUNT_ATTR)])
        .await
        .context("Failed to search for account ID")?;

    match (token_items.first(), account_items.first()) {
        (Some(token_item), Some(account_item)) => {
            let access_token = String::from_utf8(token_item.secret().await.context("Failed to read token secret")?.to_vec())
                .context("Token is not valid UTF-8")?;
            let account_id = String::from_utf8(account_item.secret().await.context("Failed to read account secret")?.to_vec())
                .context("Account ID is not valid UTF-8")?;

            if access_token.is_empty() || account_id.is_empty() {
                Ok(None)
            } else {
                Ok(Some(Credentials {
                    access_token,
                    account_id,
                }))
            }
        }
        _ => Ok(None),
    }
}

pub async fn delete_credentials() -> Result<()> {
    let keyring = Keyring::new().await.context("Failed to open keyring")?;

    let token_items = keyring
        .search_items(&vec![("application", APP_LABEL), ("type", TOKEN_ATTR)])
        .await
        .unwrap_or_default();

    for item in token_items {
        let _ = item.delete().await;
    }

    let account_items = keyring
        .search_items(&vec![("application", APP_LABEL), ("type", ACCOUNT_ATTR)])
        .await
        .unwrap_or_default();

    for item in account_items {
        let _ = item.delete().await;
    }

    Ok(())
}
