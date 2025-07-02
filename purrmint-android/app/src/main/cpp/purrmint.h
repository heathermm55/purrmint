// Auto-generated file, do not edit manually

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * FFI Error codes
 */
typedef enum FfiError {
    Success = 0,
    NullPointer = 1,
    InvalidInput = 2,
    ServiceError = 3,
    NotInitialized = 4,
} FfiError;

/**
 * Nostr Account structure for FFI
 */
typedef struct NostrAccount {
    char *pubkey;
    char *secret_key;
    bool is_imported;
} NostrAccount;

/**
 * Create a new Nostr account
 */
struct NostrAccount *nostr_create_account(void);

/**
 * Import an existing Nostr account from secret key
 */
struct NostrAccount *nostr_import_account(const char *secret_key_str);

/**
 * Configure the mint service
 */
enum FfiError mint_configure(const char *config_json);

/**
 * Start the mint service
 */
enum FfiError mint_start(void);

/**
 * Stop the mint service
 */
enum FfiError mint_stop(void);

/**
 * Get mint information as JSON string
 */
char *mint_get_info(void);

/**
 * Get mint status as JSON string
 */
char *mint_get_status(void);

/**
 * Get current Nostr account information
 */
char *nostr_get_account(void);

/**
 * Free a C string allocated by Rust
 */
void mint_free_string(char *s);

/**
 * Free a NostrAccount structure
 */
void nostr_free_account(struct NostrAccount *account);

/**
 * Test function to verify FFI is working
 */
char *mint_test_ffi(void);
