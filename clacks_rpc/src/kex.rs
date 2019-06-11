use actix::prelude::*;
use byteorder::{BigEndian, ByteOrder, LittleEndian};
use chrono::Duration;
use clacks_crypto::{self, csrng_gen};
use clacks_crypto::symm::AuthKey;
use clacks_mtproto::{BoxedSerialize, IntoBoxed, mtproto};
use clacks_transport;
use futures::channel::oneshot;
use futures::compat::*;

use crate::{Result, client};

fn u32_bytes(n: u32) -> mtproto::bytes {
    let mut ret = vec![0u8; 4];
    BigEndian::write_u32(&mut ret, n);
    ret.into()
}

pub async fn kex(addr: Addr<client::RpcClientActor>, expires_in: Option<Duration>)
             -> Result<(AuthKey, mtproto::FutureSalt)>
{
    let nonce = csrng_gen();
    let pq = addr.send(client::CallFunction::plain(mtproto::rpc::ReqPq { nonce })).compat().await??.only();
    assert_eq!(nonce, pq.nonce);
    let server_nonce = pq.server_nonce;
    let (pq_tx, pq_rx) = oneshot::channel::<Result<(mtproto::bytes, mtproto::bytes)>>();
    rayon::spawn({
        let pq_int = BigEndian::read_u64(&pq.pq);
        move || {
            let pq = clacks_crypto::asymm::decompose_pq(pq_int)
                .map(|(p, q)| (u32_bytes(p), u32_bytes(q)));
            let _ = pq_tx.send(pq);
        }
    });
    let (pubkey, public_key_fingerprint) = clacks_crypto::asymm::find_first_key(&pq.server_public_key_fingerprints)?;
    let new_nonce = csrng_gen();
    let (p, q) = pq_rx.await??;
    let (aes, dh_params) = {
        let inner = {
            let (p, q) = (p.clone(), q.clone());
            match expires_in {
                Some(expires_in) => mtproto::p_q_inner_data::Temp {
                    nonce, new_nonce, server_nonce, p, q,
                    expires_in: {
                        let s = expires_in.num_seconds();
                        assert!(s < ::std::i32::MAX as i64);
                        s as i32
                    },
                    pq: pq.pq,
                }.into_boxed(),
                None => mtproto::p_q_inner_data::PQInnerData {
                    nonce, new_nonce, server_nonce, p, q,
                    pq: pq.pq,
                }.into_boxed(),
            }
        };
        let aes = clacks_crypto::symm::AesParams::from_pq_inner_data(&inner)?;
        let encrypted_data = pubkey.encrypt(&inner.boxed_serialized_bytes()?)?.into();
        let dh_params = addr.send(client::CallFunction::plain(mtproto::rpc::ReqDHParams {
            nonce, server_nonce, public_key_fingerprint, encrypted_data, p, q,
        })).compat().await??;
        (aes, dh_params)
    };
    let dh_params = match dh_params {
        mtproto::ServerDHParams::Ok(x) => x,
        _ => unimplemented!(),
    };
    let decrypted = aes.ige_decrypt(&dh_params.encrypted_answer)?;
    let server_dh_data = {
        let (sha_part, data_part) = decrypted.split_at(20);
        let mut bytes = data_part;
        let data: mtproto::ServerDHInnerData = ::clacks_mtproto::Deserializer::new(&mut bytes).read_boxed()?;
        let actual_data_part = &data_part[..data_part.len() - bytes.len()];
        let data_hash = clacks_crypto::sha1_bytes(&[actual_data_part])?;
        assert_eq!(sha_part, &data_hash[..]);
        data
    };
    let (auth_key, g_b) = clacks_crypto::asymm::calculate_auth_key(&server_dh_data)?;
    let dh_answer = {
        let inner = mtproto::client_dh_inner_data::ClientDHInnerData {
            nonce, server_nonce, g_b,
            retry_id: 0,
        }.into_boxed();
        let encrypted_data = aes.ige_encrypt(&inner.boxed_serialized_bytes()?, true)?.into();
        let set_dh = mtproto::rpc::SetClientDHParams { nonce, server_nonce, encrypted_data };
        addr.send(client::CallFunction::plain(set_dh)).compat().await??
    };
    let expected_new_nonce_hash1 = auth_key.new_nonce_hash(1, new_nonce)?;
    match dh_answer {
        mtproto::SetClientDHParamsAnswer::Ok(ref n) if n.new_nonce_hash1 == expected_new_nonce_hash1 => (),
        _ => unimplemented!(),
    }
    let mut new_salt = [0u8; 8];
    for ((loc, &a), &b) in new_salt.iter_mut().zip(&new_nonce[..8]).zip(&server_nonce[..8]) {
        *loc = a ^ b;
    }
    let salt = clacks_transport::session::future_salt_from_negotiated_salt(LittleEndian::read_i64(&new_salt));
    Ok((auth_key, salt))
}

pub async fn new_auth_key(addr: Addr<client::RpcClientActor>, temp_key_duration: Duration) -> Result<AuthKey>
{
    let (temp_key, salt) = kex(addr.clone(), Some(temp_key_duration)).await?;
    let (perm_key, _) = kex(addr.clone(), None).await?;
    addr.send(client::BindAuthKey {
        temp_key, temp_key_duration, salt,
        perm_key: perm_key.clone(),
    }).compat().await??;
    Ok(perm_key)
}

pub async fn adopt_auth_key(addr: Addr<client::RpcClientActor>, temp_key_duration: Duration, perm_key: AuthKey) -> Result<()>
{
    let (temp_key, salt) = kex(addr.clone(), Some(temp_key_duration)).await?;
    addr.send(client::BindAuthKey {
        temp_key, temp_key_duration, salt, perm_key,
    }).compat().await??;
    Ok(())
}
