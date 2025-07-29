use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use jni::{
    JNIEnv, objects::{JClass, JString, JObject},
    sys::{jboolean, jlong, jstring, jobjectArray, jobject},
};
use anyhow::Result;
use log::error;

use crate::lightning_wallet::{LightningWallet, WalletStatus, WalletBalance, ChannelInfo, PaymentInfo};

/// Global Lightning wallet instance
static mut LIGHTNING_WALLET: Option<Arc<Mutex<LightningWallet>>> = None;

/// Initialize Lightning wallet
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_wallet_LightningWalletManager_nativeInitialize(
    env: JNIEnv,
    _class: JClass,
    storage_path: JString,
) -> jboolean {
    let result: Result<bool> = (|| {
        let storage_path_str: String = env.get_string(&storage_path)?.into();
        let storage_path = PathBuf::from(storage_path_str);
        
        let mut wallet = LightningWallet::new(storage_path)?;
        wallet.initialize()?;
        
        unsafe {
            LIGHTNING_WALLET = Some(Arc::new(Mutex::new(wallet)));
        }
        
        Ok(true)
    })();
    
    match result {
        Ok(_) => 1,
        Err(e) => {
            error!("Failed to initialize Lightning wallet: {}", e);
            0
        }
    }
}

/// Get wallet status
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_wallet_LightningWalletManager_nativeGetStatus(
    env: JNIEnv,
    _class: JClass,
) -> jobject {
    let result: Result<JObject> = (|| {
        unsafe {
            if let Some(wallet) = &LIGHTNING_WALLET {
                let wallet_guard = wallet.lock().unwrap();
                let status = wallet_guard.get_status()?;
                
                // Create WalletStatus object
                let status_class = env.find_class("com/purrmint/app/wallet/WalletStatus")?;
                let status_obj = env.new_object(
                    status_class,
                    "(ZLjava/lang/String;Z)V",
                    &[
                        status.is_running.into(),
                        status.node_id.map(|id| env.new_string(&id)?.into()).unwrap_or(std::ptr::null_mut()),
                        status.is_connected.into(),
                    ]
                )?;
                
                Ok(status_obj.into())
            } else {
                anyhow::bail!("Lightning wallet not initialized")
            }
        }
    })();
    
    match result {
        Ok(obj) => obj.into(),
        Err(e) => {
            error!("Failed to get wallet status: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Get wallet balance
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_wallet_LightningWalletManager_nativeGetBalance(
    env: JNIEnv,
    _class: JClass,
) -> jobject {
    let result: Result<jobject> = (|| {
        unsafe {
            if let Some(wallet) = &LIGHTNING_WALLET {
                let wallet_guard = wallet.lock().unwrap();
                if let Some(balance) = wallet_guard.get_balance()? {
                    // Create WalletBalance object
                    let balance_class = env.find_class("com/purrmint/app/wallet/WalletBalance")?;
                    let balance_obj = env.new_object(
                        balance_class,
                        "(JJ)V",
                        &[
                            jni::objects::JValueGen::Long(balance.lightning_balance_msat as jlong),
                            jni::objects::JValueGen::Long(balance.onchain_balance_sats as jlong),
                        ]
                    )?;
                    
                    Ok(balance_obj.into())
                } else {
                    Ok(std::ptr::null_mut())
                }
            } else {
                anyhow::bail!("Lightning wallet not initialized")
            }
        }
    })();
    
    match result {
        Ok(obj) => obj,
        Err(e) => {
            error!("Failed to get wallet balance: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Get channels list
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_wallet_LightningWalletManager_nativeGetChannels(
    env: JNIEnv,
    _class: JClass,
) -> jobjectArray {
    let result: Result<jobjectArray> = (|| {
        unsafe {
            if let Some(wallet) = &LIGHTNING_WALLET {
                let wallet_guard = wallet.lock().unwrap();
                let channels = wallet_guard.get_channels()?;
                
                // Create ChannelInfo array
                let channel_class = env.find_class("com/purrmint/app/wallet/ChannelInfo")?;
                let array = env.new_object_array(
                    channels.len() as i32,
                    channel_class,
                    std::ptr::null_mut()
                )?;
                
                for (i, channel) in channels.iter().enumerate() {
                    let channel_obj = env.new_object(
                        channel_class,
                        "(Ljava/lang/String;Ljava/lang/String;JJLjava/lang/String;)V",
                        &[
                            env.new_string(&channel.channel_id)?.into(),
                            env.new_string(&channel.peer_id)?.into(),
                            channel.capacity_msat as jlong,
                            channel.balance_msat as jlong,
                            env.new_string(&channel.status)?.into(),
                        ]
                    )?;
                    
                    env.set_object_array_element(&array, i as i32, &channel_obj)?;
                }
                
                Ok(array.into_inner())
            } else {
                anyhow::bail!("Lightning wallet not initialized")
            }
        }
    })();
    
    match result {
        Ok(array) => array,
        Err(e) => {
            error!("Failed to get channels: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Get payments list
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_wallet_LightningWalletManager_nativeGetPayments(
    env: JNIEnv,
    _class: JClass,
) -> jobjectArray {
    let result: Result<jobjectArray> = (|| {
        unsafe {
            if let Some(wallet) = &LIGHTNING_WALLET {
                let wallet_guard = wallet.lock().unwrap();
                let payments = wallet_guard.get_payments()?;
                
                // Create PaymentInfo array
                let payment_class = env.find_class("com/purrmint/app/wallet/PaymentInfo")?;
                let array = env.new_object_array(
                    payments.len() as i32,
                    payment_class,
                    std::ptr::null_mut()
                )?;
                
                for (i, payment) in payments.iter().enumerate() {
                    let payment_obj = env.new_object(
                        payment_class,
                        "(Ljava/lang/String;JLjava/lang/String;Ljava/lang/String;ZJ)V",
                        &[
                            env.new_string(&payment.payment_id)?.into(),
                            payment.amount_msat as jlong,
                            env.new_string(&payment.description)?.into(),
                            env.new_string(&payment.status)?.into(),
                            payment.is_incoming.into(),
                            payment.timestamp as jlong,
                        ]
                    )?;
                    
                    env.set_object_array_element(&array, i as i32, &payment_obj)?;
                }
                
                Ok(array.into_inner())
            } else {
                anyhow::bail!("Lightning wallet not initialized")
            }
        }
    })();
    
    match result {
        Ok(array) => array,
        Err(e) => {
            error!("Failed to get payments: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Create invoice
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_wallet_LightningWalletManager_nativeCreateInvoice(
    env: JNIEnv,
    _class: JClass,
    amount_sats: jlong,
    description: JString,
) -> jstring {
    let result: Result<jstring> = (|| {
        let description_str: String = env.get_string(&description)?.into();
        
        unsafe {
            if let Some(wallet) = &LIGHTNING_WALLET {
                let wallet_guard = wallet.lock().unwrap();
                let invoice = wallet_guard.create_invoice(amount_sats as u64, description_str)?;
                
                Ok(env.new_string(&invoice)?.into_inner())
            } else {
                anyhow::bail!("Lightning wallet not initialized")
            }
        }
    })();
    
    match result {
        Ok(invoice) => invoice,
        Err(e) => {
            error!("Failed to create invoice: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Pay invoice
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_wallet_LightningWalletManager_nativePayInvoice(
    env: JNIEnv,
    _class: JClass,
    invoice: JString,
) -> jstring {
    let result: Result<jstring> = (|| {
        let invoice_str: String = env.get_string(&invoice)?.into();
        
        unsafe {
            if let Some(wallet) = &LIGHTNING_WALLET {
                let wallet_guard = wallet.lock().unwrap();
                let payment_id = wallet_guard.pay_invoice(invoice_str)?;
                
                Ok(env.new_string(&payment_id)?.into_inner())
            } else {
                anyhow::bail!("Lightning wallet not initialized")
            }
        }
    })();
    
    match result {
        Ok(payment_id) => payment_id,
        Err(e) => {
            error!("Failed to pay invoice: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Open channel
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_wallet_LightningWalletManager_nativeOpenChannel(
    env: JNIEnv,
    _class: JClass,
    node_id: JString,
    address: JString,
    amount_sats: jlong,
) -> jstring {
    let result: Result<jstring> = (|| {
        let node_id_str: String = env.get_string(&node_id)?.into();
        let address_str: String = env.get_string(&address)?.into();
        
        unsafe {
            if let Some(wallet) = &LIGHTNING_WALLET {
                let wallet_guard = wallet.lock().unwrap();
                let channel_id = wallet_guard.open_channel(node_id_str, address_str, amount_sats as u64)?;
                
                Ok(env.new_string(&channel_id)?.into_inner())
            } else {
                anyhow::bail!("Lightning wallet not initialized")
            }
        }
    })();
    
    match result {
        Ok(channel_id) => channel_id,
        Err(e) => {
            error!("Failed to open channel: {}", e);
            std::ptr::null_mut()
        }
    }
}

/// Cleanup Lightning wallet
#[no_mangle]
pub extern "system" fn Java_com_purrmint_app_wallet_LightningWalletManager_nativeCleanup(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    let result: Result<bool> = (|| {
        unsafe {
            if let Some(wallet) = &LIGHTNING_WALLET {
                let mut wallet_guard = wallet.lock().unwrap();
                wallet_guard.cleanup()?;
                LIGHTNING_WALLET = None;
            }
            Ok(true)
        }
    })();
    
    match result {
        Ok(_) => true,
        Err(e) => {
            error!("Failed to cleanup Lightning wallet: {}", e);
            false
        }
    }
} 